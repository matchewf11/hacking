use std::{collections::HashMap, time::Instant};

use reqwest::{Client, Method};
use serde_json::Value;

use crate::{
    Error,
    parser::{
        json::parse_json,
        suite::{
            Assert, AssertTarget, AssertValue, BodyTarget, LengthArg, Matcher, PathSegment, Suite,
            Test,
        },
    },
};

// ── Response data ─────────────────────────────────────────────────────────────

struct ResponseData {
    status: u16,
    headers: HashMap<String, String>,
    cookies: HashMap<String, ParsedCookie>,
    body_raw: String,
    body_json: Option<Value>,
    duration_ms: u128,
}

struct ParsedCookie {
    value: String,
    attributes: HashMap<String, Option<String>>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub async fn run_tests(suite: Suite) -> Result<(), Error> {
    let client = Client::new();
    let base = std::env::var("HURL_BASE_URL").unwrap_or_default();

    let mut total = 0usize;
    let mut passed = 0usize;

    // TODO(RFC-002): run groups concurrently with tokio::spawn; tests within a
    // group must remain sequential so later tests can depend on earlier state.
    for group in &suite.0 {
        println!("\ngroup {}", group.name);
        for test in &group.tests {
            total += 1;
            match run_test(&client, &base, test).await {
                Ok(()) => {
                    println!("  ✓  {}", test.name);
                    passed += 1;
                }
                Err(failures) => {
                    println!("  ✗  {}", test.name);
                    for msg in &failures {
                        println!("       {msg}");
                    }
                }
            }
        }
    }

    let failed = total - passed;
    println!("\n{passed} passed, {failed} failed");

    if failed > 0 {
        Err(Error::TestsFailed(failed))
    } else {
        Ok(())
    }
}

// ── Single test ───────────────────────────────────────────────────────────────

async fn run_test(client: &Client, base: &str, test: &Test) -> Result<(), Vec<String>> {
    let (method, path, body) = parse_request(&test.value)
        .map_err(|e| vec![format!("request parse error: {e}")])?;

    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path
    } else {
        format!("{base}{path}")
    };

    let started = Instant::now();

    let mut req = client.request(method, &url);
    if let Some(body_json) = body {
        req = req
            .header("Content-Type", "application/json")
            .body(body_json.to_string());
    }

    let resp = req
        .send()
        .await
        .map_err(|e| vec![format!("request failed: {e}")])?;

    let duration_ms = started.elapsed().as_millis();
    let status = resp.status().as_u16();

    // Collect headers before consuming body
    let mut headers: HashMap<String, String> = HashMap::new();
    for (k, v) in resp.headers() {
        headers.insert(
            k.as_str().to_lowercase(),
            v.to_str().unwrap_or("").to_string(),
        );
    }

    let set_cookie_headers: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(String::from))
        .collect();

    let mut cookies: HashMap<String, ParsedCookie> = HashMap::new();
    for raw in &set_cookie_headers {
        let (name, cookie) = parse_set_cookie(raw);
        cookies.insert(name, cookie);
    }

    let body_raw = resp
        .text()
        .await
        .map_err(|e| vec![format!("failed to read body: {e}")])?;

    let body_json = serde_json::from_str::<Value>(&body_raw).ok();

    let response = ResponseData {
        status,
        headers,
        cookies,
        body_raw,
        body_json,
        duration_ms,
    };

    // Evaluate all asserts; collect all failures rather than short-circuiting
    let failures: Vec<String> = test
        .asserts
        .iter()
        .filter_map(|a| evaluate_assert(a, &response).err())
        .collect();

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

// ── Request parsing ───────────────────────────────────────────────────────────

/// Parse a test's value string into (method, path, optional JSON body).
///
/// Value format: `<method> "<path>" [body.<key> = <value> ...]`
fn parse_request(value: &str) -> Result<(Method, String, Option<Value>), String> {
    let tokens = tokenize(value);
    let mut iter = tokens.iter();

    let method_str = iter.next().ok_or("missing method")?;
    let path = iter.next().ok_or("missing path")?.clone();

    let method = match method_str.to_lowercase().as_str() {
        "get" => Method::GET,
        "post" => Method::POST,
        "put" => Method::PUT,
        "patch" => Method::PATCH,
        "delete" => Method::DELETE,
        other => return Err(format!("unknown method: {other}")),
    };

    // Parse body params: `body.<key> = <value>` triplets
    let rest: Vec<&str> = iter.map(|s| s.as_str()).collect();
    let body = parse_body_params(&rest);

    Ok((method, path, body))
}

/// Re-tokenize a value string, stripping quotes from quoted tokens.
fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();

    loop {
        while chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            chars.next();
        }
        match chars.peek() {
            None => break,
            Some(&'"') => {
                chars.next(); // consume opening "
                let mut tok = String::new();
                loop {
                    match chars.next() {
                        None | Some('"') => break,
                        Some('\\') => {
                            if let Some(c) = chars.next() {
                                tok.push(c);
                            }
                        }
                        Some(c) => tok.push(c),
                    }
                }
                tokens.push(tok);
            }
            _ => {
                let mut tok = String::new();
                while chars.peek().map(|c| !c.is_whitespace()).unwrap_or(false) {
                    tok.push(chars.next().unwrap());
                }
                if !tok.is_empty() {
                    tokens.push(tok);
                }
            }
        }
    }

    tokens
}

/// Parse `body.<key> = <value>` triplets into a JSON object.
fn parse_body_params(tokens: &[&str]) -> Option<Value> {
    let mut pairs: Vec<String> = Vec::new();
    let mut i = 0;

    while i + 2 <= tokens.len() {
        if tokens[i + 1] == "=" {
            // Strip the `body.` prefix if present
            let key = tokens[i].strip_prefix("body.").unwrap_or(tokens[i]);
            let val = tokens[i + 2];
            pairs.push(format!("{key}={val}"));
            i += 3;
        } else {
            i += 1;
        }
    }

    if pairs.is_empty() {
        None
    } else {
        Some(parse_json(
            &pairs.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        ))
    }
}

// ── Cookie parsing ────────────────────────────────────────────────────────────

/// Parse a raw `Set-Cookie` header value.
fn parse_set_cookie(raw: &str) -> (String, ParsedCookie) {
    let mut parts = raw.splitn(2, ';');
    let name_val = parts.next().unwrap_or("").trim();
    let attrs_raw = parts.next().unwrap_or("");

    let (name, value) = name_val
        .split_once('=')
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .unwrap_or_else(|| (name_val.to_string(), String::new()));

    let mut attributes: HashMap<String, Option<String>> = HashMap::new();
    for part in attrs_raw.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((k, v)) = part.split_once('=') {
            attributes.insert(k.trim().to_lowercase(), Some(v.trim().to_string()));
        } else {
            attributes.insert(part.to_lowercase(), None);
        }
    }

    (name, ParsedCookie { value, attributes })
}

// ── Assertion evaluation ──────────────────────────────────────────────────────

fn evaluate_assert(assert: &Assert, resp: &ResponseData) -> Result<(), String> {
    let inner = check_assert(assert, resp);

    if assert.negated {
        match inner {
            Ok(()) => Err(format!(
                "assert not {} — expected to fail but passed",
                target_label(&assert.target)
            )),
            Err(_) => Ok(()),
        }
    } else {
        inner
    }
}

fn check_assert(assert: &Assert, resp: &ResponseData) -> Result<(), String> {
    match &assert.target {
        AssertTarget::Status => check_status(resp.status, &assert.matcher),
        AssertTarget::Body(bt) => check_body(bt, &assert.matcher, resp),
        AssertTarget::Header(name) => check_header(name, &assert.matcher, resp),
        AssertTarget::Cookie(name, attr) => {
            check_cookie(name, attr.as_deref(), &assert.matcher, resp)
        }
        AssertTarget::Duration => check_duration(resp.duration_ms, &assert.matcher),
    }
}

// ── Status ────────────────────────────────────────────────────────────────────

fn check_status(status: u16, matcher: &Option<Matcher>) -> Result<(), String> {
    let val = Value::Number(status.into());
    match matcher {
        // No matcher: any 2xx is truthy
        None => {
            if (200..300).contains(&status) {
                Ok(())
            } else {
                Err(format!("assert status — expected 2xx, got {status}"))
            }
        }
        Some(m) => apply_matcher("status", &val, m),
    }
}

// ── Body ──────────────────────────────────────────────────────────────────────

fn check_body(target: &BodyTarget, matcher: &Option<Matcher>, resp: &ResponseData) -> Result<(), String> {
    match target {
        BodyTarget::Raw => {
            match matcher {
                None => {
                    if !resp.body_raw.is_empty() {
                        Ok(())
                    } else {
                        Err("assert body — body is empty".to_string())
                    }
                }
                Some(Matcher::Empty) => {
                    if resp.body_raw.is_empty() {
                        Ok(())
                    } else {
                        Err("assert body empty — body is not empty".to_string())
                    }
                }
                Some(Matcher::Length(len_arg)) => {
                    let len_val = Value::Number(resp.body_raw.len().into());
                    check_length("body", &len_val, len_arg)
                }
                Some(m) => apply_matcher("body", &Value::String(resp.body_raw.clone()), m),
            }
        }

        BodyTarget::Path(segs) => {
            let label = format!("body.{}", display_path(segs));
            let json = resp
                .body_json
                .as_ref()
                .ok_or_else(|| format!("{label} — body is not valid JSON"))?;

            let val = navigate_json(json, segs);

            match matcher {
                // No matcher: key must exist and be truthy
                None => match val {
                    None => Err(format!("{label} — key not found")),
                    Some(Value::Null) => Err(format!("{label} — value is null")),
                    Some(Value::Bool(false)) => Err(format!("{label} — value is false")),
                    Some(Value::String(s)) if s.is_empty() => {
                        Err(format!("{label} — value is empty string"))
                    }
                    _ => Ok(()),
                },
                Some(Matcher::Present) => match val {
                    Some(v) if !v.is_null() => Ok(()),
                    _ => Err(format!("{label} — not present")),
                },
                Some(Matcher::Absent) => match val {
                    None | Some(Value::Null) => Ok(()),
                    Some(v) => Err(format!("{label} — expected absent, got {v}")),
                },
                Some(Matcher::Length(len_arg)) => {
                    let v = val.ok_or_else(|| format!("{label} — key not found"))?;
                    let len = match v {
                        Value::String(s) => s.len(),
                        Value::Array(a) => a.len(),
                        _ => return Err(format!("{label} length — expected string or array")),
                    };
                    check_length(&label, &Value::Number(len.into()), len_arg)
                }
                Some(m) => {
                    let v = val.ok_or_else(|| format!("{label} — key not found"))?;
                    apply_matcher(&label, v, m)
                }
            }
        }
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

fn check_header(name: &str, matcher: &Option<Matcher>, resp: &ResponseData) -> Result<(), String> {
    let label = format!("header.{name}");
    let val = resp.headers.get(name).map(|s| s.as_str());

    match matcher {
        None | Some(Matcher::Present) => match val {
            Some(_) => Ok(()),
            None => Err(format!("{label} — not present")),
        },
        Some(Matcher::Absent) => match val {
            None => Ok(()),
            Some(v) => Err(format!("{label} — expected absent, got {v:?}")),
        },
        Some(m) => {
            let v = val.ok_or_else(|| format!("{label} — header not found"))?;
            apply_matcher(&label, &Value::String(v.to_string()), m)
        }
    }
}

// ── Cookie ────────────────────────────────────────────────────────────────────

fn check_cookie(
    name: &str,
    attr: Option<&str>,
    matcher: &Option<Matcher>,
    resp: &ResponseData,
) -> Result<(), String> {
    let cookie = resp.cookies.get(name);

    match attr {
        None => {
            let label = format!("cookie.{name}");
            match matcher {
                None | Some(Matcher::Present) => match cookie {
                    Some(_) => Ok(()),
                    None => Err(format!("{label} — not set")),
                },
                Some(Matcher::Absent) => match cookie {
                    None => Ok(()),
                    Some(_) => Err(format!("{label} — expected absent but was set")),
                },
                Some(m) => {
                    let c = cookie.ok_or_else(|| format!("{label} — not set"))?;
                    apply_matcher(&label, &Value::String(c.value.clone()), m)
                }
            }
        }

        Some(attr_name) => {
            let label = format!("cookie.{name}.{attr_name}");
            let c = cookie.ok_or_else(|| format!("cookie.{name} — not set"))?;
            let attr_val = c.attributes.get(attr_name);

            match matcher {
                // No matcher or `present` — just check the attribute exists (e.g. httponly, secure)
                None | Some(Matcher::Present) => match attr_val {
                    Some(_) => Ok(()),
                    None => Err(format!("{label} — attribute not set")),
                },
                Some(Matcher::Absent) => match attr_val {
                    None => Ok(()),
                    Some(_) => Err(format!("{label} — expected absent but was set")),
                },
                Some(m) => {
                    let v = attr_val
                        .ok_or_else(|| format!("{label} — attribute not set"))?
                        .clone()
                        .ok_or_else(|| format!("{label} — attribute has no value (flag only)"))?;
                    apply_matcher(&label, &Value::String(v), m)
                }
            }
        }
    }
}

// ── Duration ──────────────────────────────────────────────────────────────────

fn check_duration(ms: u128, matcher: &Option<Matcher>) -> Result<(), String> {
    let val = Value::Number(serde_json::Number::from(ms as u64));
    match matcher {
        None => Ok(()),
        Some(m) => apply_matcher("duration", &val, m),
    }
}

// ── Shared matcher logic ──────────────────────────────────────────────────────

fn apply_matcher(label: &str, actual: &Value, matcher: &Matcher) -> Result<(), String> {
    match matcher {
        Matcher::Present => Ok(()),
        Matcher::Absent => Err(format!("{label} — expected absent, got {actual}")),
        Matcher::Empty => match actual {
            Value::String(s) if s.is_empty() => Ok(()),
            Value::Array(a) if a.is_empty() => Ok(()),
            _ => Err(format!("{label} empty — got {actual}")),
        },
        Matcher::Equals(expected) => {
            if json_matches(actual, expected) {
                Ok(())
            } else {
                Err(format!(
                    "assert {label} equals {}\n  expected: {}\n       got: {actual}",
                    display_assert_value(expected),
                    display_assert_value(expected),
                ))
            }
        }
        Matcher::NotEquals(expected) => {
            if !json_matches(actual, expected) {
                Ok(())
            } else {
                Err(format!(
                    "assert {label} not-equals {} — values were equal",
                    display_assert_value(expected),
                ))
            }
        }
        Matcher::Contains(expected) => {
            let needle = assert_value_to_str(expected)
                .ok_or_else(|| format!("{label} contains — value must be a string"))?;
            match actual {
                Value::String(s) => {
                    if s.contains(needle.as_str()) {
                        Ok(())
                    } else {
                        Err(format!(
                            "assert {label} contains {needle:?}\n  expected: to contain {needle:?}\n       got: {s:?}"
                        ))
                    }
                }
                Value::Array(arr) => {
                    let sv = Value::String(needle.clone());
                    if arr.contains(&sv) {
                        Ok(())
                    } else {
                        Err(format!("{label} contains {needle:?} — not found in array"))
                    }
                }
                _ => Err(format!("{label} contains — not a string or array")),
            }
        }
        Matcher::NotContains(expected) => {
            let needle = assert_value_to_str(expected)
                .ok_or_else(|| format!("{label} not-contains — value must be a string"))?;
            match actual {
                Value::String(s) => {
                    if !s.contains(needle.as_str()) {
                        Ok(())
                    } else {
                        Err(format!("{label} not-contains {needle:?} — found in string"))
                    }
                }
                _ => Err(format!("{label} not-contains — not a string")),
            }
        }
        Matcher::StartsWith(expected) => {
            let prefix = assert_value_to_str(expected)
                .ok_or_else(|| format!("{label} starts-with — value must be a string"))?;
            match actual {
                Value::String(s) => {
                    if s.starts_with(prefix.as_str()) {
                        Ok(())
                    } else {
                        Err(format!(
                            "assert {label} starts-with {prefix:?}\n  expected: starts with {prefix:?}\n       got: {s:?}"
                        ))
                    }
                }
                _ => Err(format!("{label} starts-with — not a string")),
            }
        }
        Matcher::EndsWith(expected) => {
            let suffix = assert_value_to_str(expected)
                .ok_or_else(|| format!("{label} ends-with — value must be a string"))?;
            match actual {
                Value::String(s) => {
                    if s.ends_with(suffix.as_str()) {
                        Ok(())
                    } else {
                        Err(format!(
                            "assert {label} ends-with {suffix:?}\n  expected: ends with {suffix:?}\n       got: {s:?}"
                        ))
                    }
                }
                _ => Err(format!("{label} ends-with — not a string")),
            }
        }
        Matcher::Gt(e) => compare_nums(label, actual, e, "gt", |a, b| a > b),
        Matcher::Gte(e) => compare_nums(label, actual, e, "gte", |a, b| a >= b),
        Matcher::Lt(e) => compare_nums(label, actual, e, "lt", |a, b| a < b),
        Matcher::Lte(e) => compare_nums(label, actual, e, "lte", |a, b| a <= b),
        Matcher::Length(len_arg) => check_length(label, actual, len_arg),
        Matcher::Matches(_pattern) => {
            // TODO: add the `regex` crate and implement this
            Err(format!("{label} matches — regex not yet implemented"))
        }
    }
}

fn check_length(label: &str, actual: &Value, arg: &LengthArg) -> Result<(), String> {
    let len = match actual {
        Value::String(s) => s.len(),
        Value::Array(a) => a.len(),
        Value::Number(n) => n.as_u64().unwrap_or(0) as usize, // pre-computed length
        _ => return Err(format!("{label} length — not a string or array")),
    };
    let len_val = Value::Number(len.into());
    match arg {
        LengthArg::Exact(e) => {
            let exp = assert_value_to_f64(e)
                .ok_or_else(|| format!("{label} length — expected must be a number"))?;
            if (len as f64) == exp {
                Ok(())
            } else {
                Err(format!("assert {label} length {exp} — got {len}"))
            }
        }
        LengthArg::Gt(e) => compare_nums(label, &len_val, e, "length gt", |a, b| a > b),
        LengthArg::Gte(e) => compare_nums(label, &len_val, e, "length gte", |a, b| a >= b),
        LengthArg::Lt(e) => compare_nums(label, &len_val, e, "length lt", |a, b| a < b),
        LengthArg::Lte(e) => compare_nums(label, &len_val, e, "length lte", |a, b| a <= b),
    }
}

fn compare_nums(
    label: &str,
    actual: &Value,
    expected: &AssertValue,
    op: &str,
    cmp: impl Fn(f64, f64) -> bool,
) -> Result<(), String> {
    let a = actual
        .as_f64()
        .ok_or_else(|| format!("{label} {op} — not a number: {actual}"))?;
    let b = assert_value_to_f64(expected)
        .ok_or_else(|| format!("{label} {op} — expected must be a number"))?;
    if cmp(a, b) {
        Ok(())
    } else {
        Err(format!("assert {label} {op} {b} — got {a}"))
    }
}

// ── JSON helpers ──────────────────────────────────────────────────────────────

fn navigate_json<'a>(root: &'a Value, path: &[PathSegment]) -> Option<&'a Value> {
    let mut cur = root;
    for seg in path {
        cur = match seg {
            PathSegment::Key(k) => cur.get(k)?,
            PathSegment::Index(i) => cur.get(i)?,
        };
    }
    Some(cur)
}

fn json_matches(actual: &Value, expected: &AssertValue) -> bool {
    match expected {
        AssertValue::String(s) => actual.as_str().map(|a| a == s).unwrap_or(false),
        AssertValue::Number(n) => actual.as_f64().map(|a| a == *n).unwrap_or(false),
        AssertValue::Bool(b) => actual.as_bool().map(|a| a == *b).unwrap_or(false),
        AssertValue::Null => actual.is_null(),
        AssertValue::Regex(_) => false,
    }
}

fn assert_value_to_f64(v: &AssertValue) -> Option<f64> {
    match v {
        AssertValue::Number(n) => Some(*n),
        _ => None,
    }
}

fn assert_value_to_str(v: &AssertValue) -> Option<String> {
    match v {
        AssertValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn display_path(segs: &[PathSegment]) -> String {
    segs.iter()
        .enumerate()
        .map(|(i, s)| match s {
            PathSegment::Key(k) => {
                if i == 0 {
                    k.clone()
                } else {
                    format!(".{k}")
                }
            }
            PathSegment::Index(n) => format!("[{n}]"),
        })
        .collect()
}

fn display_assert_value(v: &AssertValue) -> String {
    match v {
        AssertValue::String(s) => format!("{s:?}"),
        AssertValue::Number(n) => n.to_string(),
        AssertValue::Bool(b) => b.to_string(),
        AssertValue::Null => "null".to_string(),
        AssertValue::Regex(r) => format!("r\"{r}\""),
    }
}

fn target_label(t: &AssertTarget) -> String {
    match t {
        AssertTarget::Status => "status".to_string(),
        AssertTarget::Body(BodyTarget::Raw) => "body".to_string(),
        AssertTarget::Body(BodyTarget::Path(segs)) => format!("body.{}", display_path(segs)),
        AssertTarget::Header(n) => format!("header.{n}"),
        AssertTarget::Cookie(n, None) => format!("cookie.{n}"),
        AssertTarget::Cookie(n, Some(a)) => format!("cookie.{n}.{a}"),
        AssertTarget::Duration => "duration".to_string(),
    }
}
