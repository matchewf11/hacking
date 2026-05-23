use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn preproc(input: &str) -> String {
    let (cleaned, map) = collect_replacements(input);
    let replaced = apply_replacements(&cleaned, &map);
    expand_includes(&replaced, std::path::Path::new("."))
}

fn expand_includes(input: &str, base: &Path) -> String {
    let mut out = String::new();

    for line in input.lines() {
        let line = line.trim();

        if let Some(path) = parse_include(line) {
            let full_path = base.join(path);

            let content = fs::read_to_string(&full_path)
                .unwrap_or_else(|e| panic!("failed to read {:?}: {}", full_path, e));

            let dir = full_path.parent().unwrap_or(base);

            out.push_str(&expand_includes(&content, dir));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

fn parse_include(line: &str) -> Option<&str> {
    let line = line.trim();
    line.strip_prefix("include \"")
        .and_then(|s| s.strip_suffix('"'))
}

fn collect_replacements(input: &str) -> (String, HashMap<String, String>) {
    let mut map = HashMap::new();
    let mut out = String::new();

    for line in input.lines() {
        let line = line.trim();

        if let Some(rest) = line.strip_prefix("replace ") {
            let mut parts = rest.splitn(2, char::is_whitespace);

            let key = parts.next().unwrap().to_string();
            let value = parts
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| env::var(&key).unwrap_or_default());

            map.insert(key, value);
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    (out, map)
}

fn apply_replacements(input: &str, map: &HashMap<String, String>) -> String {
    let mut out = input.to_string();

    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    for k in keys {
        let v = &map[k];
        if out.contains(k) {
            out = out.replace(k, v);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_env(key: &str, value: &str, f: impl FnOnce()) {
        unsafe {
            std::env::set_var(key, value);
        }

        f();

        unsafe {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn replaces_literal_values() {
        let input = r#"
replace FOO bar

hello FOO world
"#;

        let out = preproc(input);

        assert!(out.contains("hello bar world"));
        assert!(!out.contains("FOO"));
    }

    #[test]
    fn replaces_multiple_occurrences() {
        let input = r#"
replace FOO x

FOO FOO FOO
"#;

        let out = preproc(input);

        assert!(out.contains("x x x"));
    }

    #[test]
    fn uses_env_fallback_when_no_value_given() {
        with_env("FOO", "env_value", || {
            let input = r#"
replace FOO

hello FOO
"#;

            let out = preproc(input);

            assert!(out.contains("hello env_value"));
        });
    }

    #[test]
    fn explicit_value_overrides_env() {
        with_env("FOO", "env_value", || {
            let input = r#"
replace FOO override

FOO
"#;

            let out = preproc(input);

            assert!(out.contains("override"));
            assert!(!out.contains("env_value"));
        });
    }

    #[test]
    fn ignores_unknown_tokens() {
        let input = r#"
replace FOO bar

this should stay unchanged
"#;

        let out = preproc(input);

        assert!(out.contains("this should stay unchanged"));
    }

    #[test]
    fn does_not_crash_on_no_replacements() {
        let input = r#"
hello world
"#;

        let out = preproc(input);

        assert_eq!(out.trim(), "hello world");
    }
}
