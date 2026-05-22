use serde_json::{Value, json, Map};

fn parse_json(inputs: &[&str]) -> Value {
    let val = Map::new();

    for input in inputs {



    }

    Value::Object(val)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(json, json![{
            "foo": "bar",
            "tbl": {
                "jim": "mmy",
                "baz": "bax"
            },
            "a": 30,
            "b": ["foo", "bar", 1, true, false, null]
        }]);
    }
}
