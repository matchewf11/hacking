use serde_json::{Map, Value};

struct Suite {
    groups: Vec<Group>,
}

struct Group {
    name: String,
    tests: Vec<Test>,
}

struct Test {
    name: String,
    value: String,
    asserts: Vec<String>,
}

pub fn parse_suite() -> Suite {
    todo!();
}

pub fn parse_json(inputs: &[&str]) -> Value {
    let mut val = Map::new();
    for input in inputs {
        match input.split_once('=') {
            None => todo!("not a json thing, only one element"),
            Some((k, v)) => {
                insert_nested(&mut val, k, parse_value(v));
            }
        }
    }
    Value::Object(val)
}

fn insert_nested(map: &mut Map<String, Value>, key: &str, value: Value) {
    let parts: Vec<&str> = key.split('.').collect();

    let mut current = map;

    for i in 0..parts.len() {
        let part = parts[i];

        if i == parts.len() - 1 {
            current.insert(part.to_string(), value);
            return;
        }

        current = current
            .entry(part.to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("expected object");
    }
}

fn parse_value(input: &str) -> Value {
    if let Ok(n) = input.parse::<i32>() {
        return n.into();
    }

    match input {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        "null" => return Value::Null,
        _ => (),
    }

    if input.starts_with('[') {
        return parse_arr(input);
    }

    input.into()
}

fn parse_arr(input: &str) -> Value {
    let inner = input
        .trim()
        .strip_prefix('[')
        .unwrap()
        .strip_suffix(']')
        .unwrap();

    let mut arr = vec![];

    for item in inner.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }

        arr.push(parse_value(item));
    }

    Value::Array(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_suite() {
        let input = r#"
group "auth"
    test "returns authenticated user"
        get "/foo";
        assert status 200;
        assert body.foo;
    end
    test "returns expected foo value"
        get "/foo";
        assert status 200;
        assert body.foo "bar";
    end
end

group "foo"
    test "returns authenticated user"
        get "/foo";
        assert status 200;
        assert body.foo;
    end
    test "returns expected foo value"
        get "/foo";
        assert status 200;
        assert body.foo "bar";
    end

    test "returns authenticated user"
        get "/foo";
        assert status 200;
        assert body.foo;
    end

end"#;

    }

    #[test]
    fn test_parse_json() {
        let input = [
            ("foo=bar"),
            ("tbl.jim=mmy"),
            ("tbl.baz=bax"),
            ("a=30"),
            ("b=[foo, bar, 1, true, false, null]"),
        ];
        let json = parse_json(&input);
        assert_eq!(
            json,
            json![{
                "foo": "bar",
                "tbl": {
                    "jim": "mmy",
                    "baz": "bax"
                },
                "a": 30,
                "b": ["foo", "bar", 1, true, false, null]
            }]
        );
    }
}
