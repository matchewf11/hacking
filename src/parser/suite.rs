use std::{iter::Peekable, str::Bytes};

// ── Tokens ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone)]
enum Token {
    // Structure keywords
    End,
    Group,
    Test,
    Assert,
    Not,
    // Data
    Ident(String),
    QuotedString(String),
    Regex(String),
}

impl Token {
    fn from_ident(s: String) -> Self {
        match s.as_str() {
            "end" => Token::End,
            "group" => Token::Group,
            "test" => Token::Test,
            "assert" => Token::Assert,
            "not" => Token::Not,
            _ => Token::Ident(s),
        }
    }
}

// ── Lexer ─────────────────────────────────────────────────────────────────────

struct Lexer<'a>(Peekable<Bytes<'a>>);

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer(input.bytes().peekable())
    }

    fn read_quoted(&mut self) -> Token {
        let mut s = Vec::new();
        loop {
            match self.0.next() {
                None => break,
                Some(b'"') => break,
                Some(b'\\') => match self.0.next() {
                    Some(b'"') => s.push(b'"'),
                    Some(b'\\') => s.push(b'\\'),
                    Some(b'n') => s.push(b'\n'),
                    Some(b't') => s.push(b'\t'),
                    Some(c) => {
                        s.push(b'\\');
                        s.push(c);
                    }
                    None => break,
                },
                Some(c) => s.push(c),
            }
        }
        Token::QuotedString(String::from_utf8(s).unwrap())
    }

    fn read_ident_tail(&mut self, first: u8) -> Token {
        let mut s = vec![first];
        while let Some(&b) = self.0.peek() {
            if b.is_ascii_whitespace() {
                break;
            }
            s.push(self.0.next().unwrap());
        }
        Token::from_ident(String::from_utf8(s).unwrap())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespace
        loop {
            match self.0.peek() {
                None => return None,
                Some(b) if b.is_ascii_whitespace() => {
                    self.0.next();
                }
                _ => break,
            }
        }

        let b = self.0.next().unwrap();

        Some(match b {
            // Quoted string
            b'"' => self.read_quoted(),

            // Regex literal r"..." or ident starting with 'r'
            b'r' => {
                if self.0.peek() == Some(&b'"') {
                    self.0.next(); // consume opening "
                    let mut s = Vec::new();
                    loop {
                        match self.0.next() {
                            None | Some(b'"') => break,
                            Some(c) => s.push(c),
                        }
                    }
                    Token::Regex(String::from_utf8(s).unwrap())
                } else {
                    self.read_ident_tail(b'r')
                }
            }

            c => self.read_ident_tail(c),
        })
    }
}

// ── AST ───────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Debug)]
pub struct Suite(pub Vec<Group>);

#[derive(PartialEq, Debug)]
pub struct Group {
    pub name: String,
    pub tests: Vec<Test>,
}

#[derive(PartialEq, Debug)]
pub struct Test {
    pub name: String,
    pub value: String,
    pub asserts: Vec<Assert>,
}

#[derive(PartialEq, Debug)]
pub struct Assert {
    pub negated: bool,
    pub target: AssertTarget,
    pub matcher: Option<Matcher>,
}

#[derive(PartialEq, Debug)]
pub enum AssertTarget {
    /// `assert status <code>`
    Status,
    /// `assert body` or `assert body.<path>`
    Body(BodyTarget),
    /// `assert header.<name>`
    Header(String),
    /// `assert cookie.<name>` or `assert cookie.<name>.<attribute>`
    Cookie(String, Option<String>),
    /// `assert duration`
    Duration,
}

#[derive(PartialEq, Debug)]
pub enum BodyTarget {
    Raw,
    Path(Vec<PathSegment>),
}

#[derive(PartialEq, Debug)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[derive(PartialEq, Debug)]
pub enum Matcher {
    // Unary
    Present,
    Absent,
    Empty,
    // Binary
    Equals(AssertValue),
    NotEquals(AssertValue),
    Contains(AssertValue),
    NotContains(AssertValue),
    StartsWith(AssertValue),
    EndsWith(AssertValue),
    Gt(AssertValue),
    Gte(AssertValue),
    Lt(AssertValue),
    Lte(AssertValue),
    Length(LengthArg),
    Matches(String),
}

/// `assert body.items length 5` vs `assert body.tags length gt 0`
#[derive(PartialEq, Debug)]
pub enum LengthArg {
    Exact(AssertValue),
    Gt(AssertValue),
    Gte(AssertValue),
    Lt(AssertValue),
    Lte(AssertValue),
}

#[derive(PartialEq, Debug)]
pub enum AssertValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Regex(String),
}

// ── Parser ────────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> Result<Suite, String> {
    Parser::new(Lexer::new(input).collect::<Vec<_>>().into_iter()).parse()
}

struct Parser<I: Iterator<Item = Token>>(Peekable<I>);

impl<I: Iterator<Item = Token>> Parser<I> {
    fn new(input: I) -> Self {
        Parser(input.peekable())
    }

    fn parse(mut self) -> Result<Suite, String> {
        let mut groups = Vec::new();
        while let Some(group) = self.parse_group()? {
            groups.push(group);
        }
        Ok(Suite(groups))
    }

    fn expect_ident(&mut self, msg: &str) -> Result<String, String> {
        match self.0.next() {
            Some(Token::Ident(s)) | Some(Token::QuotedString(s)) => Ok(s),
            other => Err(format!("{msg}, got {:?}", other)),
        }
    }

    fn parse_group(&mut self) -> Result<Option<Group>, String> {
        match self.0.peek() {
            None => return Ok(None),
            Some(Token::Group) => {
                self.0.next();
            }
            other => return Err(format!("expected `group`, got {:?}", other)),
        }

        let name = self.expect_ident("expected group name")?;
        let mut tests = Vec::new();

        loop {
            match self.0.peek() {
                Some(Token::Test) => tests.push(self.parse_test()?),
                Some(Token::End) => {
                    self.0.next();
                    break;
                }
                Some(tok) => return Err(format!("unexpected token in group: {:?}", tok)),
                None => return Err("unexpected EOF in group".to_string()),
            }
        }

        Ok(Some(Group { name, tests }))
    }

    fn parse_test(&mut self) -> Result<Test, String> {
        match self.0.next() {
            Some(Token::Test) => {}
            other => return Err(format!("expected `test`, got {:?}", other)),
        }

        let name = self.expect_ident("expected test name")?;

        // Collect request value tokens until we hit `assert` or `end`
        let mut value_parts = Vec::new();
        loop {
            match self.0.peek() {
                Some(Token::Ident(_)) => {
                    if let Some(Token::Ident(s)) = self.0.next() {
                        value_parts.push(s);
                    }
                }
                Some(Token::QuotedString(_)) => {
                    if let Some(Token::QuotedString(s)) = self.0.next() {
                        // Re-wrap so value round-trips as-written
                        value_parts.push(format!("\"{s}\""));
                    }
                }
                Some(Token::Assert) | Some(Token::End) => break,
                Some(tok) => return Err(format!("unexpected token in test value: {:?}", tok)),
                None => return Err("unexpected EOF in test value".to_string()),
            }
        }

        // Collect asserts until `end`
        let mut asserts = Vec::new();
        loop {
            match self.0.peek() {
                Some(Token::Assert) => {
                    self.0.next();
                    asserts.push(self.parse_assert()?);
                }
                Some(Token::End) => {
                    self.0.next();
                    break;
                }
                Some(tok) => return Err(format!("unexpected token in test: {:?}", tok)),
                None => return Err("unexpected EOF in test".to_string()),
            }
        }

        Ok(Test {
            name,
            value: value_parts.join(" "),
            asserts,
        })
    }

    fn parse_assert(&mut self) -> Result<Assert, String> {
        // Optional negation
        let negated = if self.0.peek() == Some(&Token::Not) {
            self.0.next();
            true
        } else {
            false
        };

        // Target must be an Ident (e.g. "status", "body.foo", "header.content-type")
        let target_tok = match self.0.next() {
            Some(Token::Ident(s)) => s,
            other => return Err(format!("expected assert target, got {:?}", other)),
        };

        let target = parse_target(&target_tok)?;
        let matcher = self.parse_matcher()?;

        Ok(Assert {
            negated,
            target,
            matcher,
        })
    }

    fn parse_matcher(&mut self) -> Result<Option<Matcher>, String> {
        match self.0.peek() {
            None | Some(Token::Assert) | Some(Token::End) => return Ok(None),
            _ => {}
        }

        let matcher = match self.0.peek() {
            Some(Token::Ident(s)) => {
                let s = s.clone();
                match s.as_str() {
                    "present" => {
                        self.0.next();
                        Matcher::Present
                    }
                    "absent" => {
                        self.0.next();
                        Matcher::Absent
                    }
                    "empty" => {
                        self.0.next();
                        Matcher::Empty
                    }
                    "equals" | "eq" => {
                        self.0.next();
                        Matcher::Equals(self.parse_value()?)
                    }
                    "not-equals" | "neq" => {
                        self.0.next();
                        Matcher::NotEquals(self.parse_value()?)
                    }
                    "contains" => {
                        self.0.next();
                        Matcher::Contains(self.parse_value()?)
                    }
                    "not-contains" => {
                        self.0.next();
                        Matcher::NotContains(self.parse_value()?)
                    }
                    "starts-with" => {
                        self.0.next();
                        Matcher::StartsWith(self.parse_value()?)
                    }
                    "ends-with" => {
                        self.0.next();
                        Matcher::EndsWith(self.parse_value()?)
                    }
                    "gt" => {
                        self.0.next();
                        Matcher::Gt(self.parse_value()?)
                    }
                    "gte" => {
                        self.0.next();
                        Matcher::Gte(self.parse_value()?)
                    }
                    "lt" => {
                        self.0.next();
                        Matcher::Lt(self.parse_value()?)
                    }
                    "lte" => {
                        self.0.next();
                        Matcher::Lte(self.parse_value()?)
                    }
                    "length" => {
                        self.0.next();
                        Matcher::Length(self.parse_length_arg()?)
                    }
                    "matches" => {
                        self.0.next();
                        Matcher::Matches(self.parse_regex()?)
                    }
                    // Not a matcher keyword → implicit `equals`
                    _ => Matcher::Equals(self.parse_value()?),
                }
            }
            // Literal value without explicit matcher keyword → implicit `equals`
            Some(Token::QuotedString(_)) | Some(Token::Regex(_)) => {
                Matcher::Equals(self.parse_value()?)
            }
            _ => return Ok(None),
        };

        Ok(Some(matcher))
    }

    fn parse_value(&mut self) -> Result<AssertValue, String> {
        match self.0.next() {
            Some(Token::QuotedString(s)) => Ok(AssertValue::String(s)),
            Some(Token::Regex(s)) => Ok(AssertValue::Regex(s)),
            Some(Token::Ident(s)) => {
                if let Ok(n) = s.parse::<f64>() {
                    return Ok(AssertValue::Number(n));
                }
                match s.as_str() {
                    "true" => Ok(AssertValue::Bool(true)),
                    "false" => Ok(AssertValue::Bool(false)),
                    "null" => Ok(AssertValue::Null),
                    _ => Ok(AssertValue::String(s)),
                }
            }
            other => Err(format!("expected value, got {:?}", other)),
        }
    }

    fn parse_length_arg(&mut self) -> Result<LengthArg, String> {
        // `length 5` | `length gt 0` | `length gte 1` etc.
        match self.0.peek() {
            Some(Token::Ident(s)) => {
                let s = s.clone();
                match s.as_str() {
                    "gt" => {
                        self.0.next();
                        Ok(LengthArg::Gt(self.parse_value()?))
                    }
                    "gte" => {
                        self.0.next();
                        Ok(LengthArg::Gte(self.parse_value()?))
                    }
                    "lt" => {
                        self.0.next();
                        Ok(LengthArg::Lt(self.parse_value()?))
                    }
                    "lte" => {
                        self.0.next();
                        Ok(LengthArg::Lte(self.parse_value()?))
                    }
                    _ => Ok(LengthArg::Exact(self.parse_value()?)),
                }
            }
            _ => Ok(LengthArg::Exact(self.parse_value()?)),
        }
    }

    fn parse_regex(&mut self) -> Result<String, String> {
        match self.0.next() {
            Some(Token::Regex(s)) => Ok(s),
            Some(Token::QuotedString(s)) => Ok(s),
            other => Err(format!("expected regex (r\"...\"), got {:?}", other)),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_target(tok: &str) -> Result<AssertTarget, String> {
    match tok {
        "status" => return Ok(AssertTarget::Status),
        "duration" => return Ok(AssertTarget::Duration),
        "body" => return Ok(AssertTarget::Body(BodyTarget::Raw)),
        _ => {}
    }

    if let Some(rest) = tok.strip_prefix("body.") {
        return Ok(AssertTarget::Body(BodyTarget::Path(parse_path(rest))));
    }
    // body[0] or body[0].foo
    if let Some(rest) = tok.strip_prefix("body") {
        if rest.starts_with('[') {
            return Ok(AssertTarget::Body(BodyTarget::Path(parse_path(rest))));
        }
    }

    if let Some(rest) = tok.strip_prefix("header.") {
        return Ok(AssertTarget::Header(rest.to_lowercase()));
    }

    if let Some(rest) = tok.strip_prefix("cookie.") {
        let (name, attr) = match rest.split_once('.') {
            Some((name, attr)) => (name.to_string(), Some(attr.to_lowercase())),
            None => (rest.to_string(), None),
        };
        return Ok(AssertTarget::Cookie(name, attr));
    }

    Err(format!("unknown assert target: {tok:?}"))
}

/// Parse a dot-separated path, handling bracket index notation.
/// e.g. `"user.name"` → `[Key("user"), Key("name")]`
///      `"items[0]"` → `[Key("items"), Index(0)]`
///      `"[0].id"` → `[Index(0), Key("id")]`
fn parse_path(path: &str) -> Vec<PathSegment> {
    // Leading bracket: [0].foo
    if path.starts_with('[') {
        if let Some(close) = path.find(']') {
            let idx_str = &path[1..close];
            let rest = &path[close + 1..];
            let rest = rest.strip_prefix('.').unwrap_or(rest);
            let mut segs = Vec::new();
            if let Ok(idx) = idx_str.parse::<usize>() {
                segs.push(PathSegment::Index(idx));
            }
            if !rest.is_empty() {
                segs.extend(parse_path(rest));
            }
            return segs;
        }
    }

    let mut segments = Vec::new();
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some((key, rest)) = part.split_once('[') {
            if !key.is_empty() {
                segments.push(PathSegment::Key(key.to_string()));
            }
            if let Some(idx_str) = rest.strip_suffix(']') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    segments.push(PathSegment::Index(idx));
                }
            }
        } else {
            segments.push(PathSegment::Key(part.to_string()));
        }
    }
    segments
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let input = r#"
        group auth
            test abc
                get "/foo"
                assert status 200
            end
        end
        "#;

        let tokens = Lexer::new(input).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::Group,
                Token::Ident("auth".to_string()),
                Token::Test,
                Token::Ident("abc".to_string()),
                Token::Ident("get".to_string()),
                Token::QuotedString("/foo".to_string()),
                Token::Assert,
                Token::Ident("status".to_string()),
                Token::Ident("200".to_string()),
                Token::End,
                Token::End,
            ],
        );
    }

    #[test]
    fn test_lexer_special_tokens() {
        let tokens = Lexer::new(r#"not r"foo" "hello world""#).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::Not,
                Token::Regex("foo".to_string()),
                Token::QuotedString("hello world".to_string()),
            ]
        );
    }

    #[test]
    fn test_parser_multiple_groups() {
        let input = r#"
        group auth
            test abc
                get "/foo"
                assert status 200
                assert body.foo
            end
        end

        group foo
            test abc
                get "/foo"
                assert status 200
                assert body.foo
            end

            test ab
                get "/foo"
                assert status 200
                assert body.foo "bar"
            end
        end
        "#;

        let suite = parse(input).unwrap();

        assert_eq!(
            suite,
            Suite(vec![
                Group {
                    name: "auth".to_string(),
                    tests: vec![Test {
                        name: "abc".to_string(),
                        value: r#"get "/foo""#.to_string(),
                        asserts: vec![
                            Assert {
                                negated: false,
                                target: AssertTarget::Status,
                                matcher: Some(Matcher::Equals(AssertValue::Number(200.0))),
                            },
                            Assert {
                                negated: false,
                                target: AssertTarget::Body(BodyTarget::Path(vec![
                                    PathSegment::Key("foo".to_string())
                                ])),
                                matcher: None,
                            },
                        ],
                    }],
                },
                Group {
                    name: "foo".to_string(),
                    tests: vec![
                        Test {
                            name: "abc".to_string(),
                            value: r#"get "/foo""#.to_string(),
                            asserts: vec![
                                Assert {
                                    negated: false,
                                    target: AssertTarget::Status,
                                    matcher: Some(Matcher::Equals(AssertValue::Number(200.0))),
                                },
                                Assert {
                                    negated: false,
                                    target: AssertTarget::Body(BodyTarget::Path(vec![
                                        PathSegment::Key("foo".to_string())
                                    ])),
                                    matcher: None,
                                },
                            ],
                        },
                        Test {
                            name: "ab".to_string(),
                            value: r#"get "/foo""#.to_string(),
                            asserts: vec![
                                Assert {
                                    negated: false,
                                    target: AssertTarget::Status,
                                    matcher: Some(Matcher::Equals(AssertValue::Number(200.0))),
                                },
                                Assert {
                                    negated: false,
                                    target: AssertTarget::Body(BodyTarget::Path(vec![
                                        PathSegment::Key("foo".to_string())
                                    ])),
                                    matcher: Some(Matcher::Equals(AssertValue::String(
                                        "bar".to_string()
                                    ))),
                                },
                            ],
                        },
                    ],
                },
            ])
        );
    }

    #[test]
    fn test_parse_assert_negation() {
        let input = r#"
        group g
            test t
                get "/x"
                assert not status 404
                assert not body.error
                assert not header.x-deprecated present
                assert not cookie.tracking present
                assert not body.user.name equals "root"
            end
        end
        "#;
        let suite = parse(input).unwrap();
        let asserts = &suite.0[0].tests[0].asserts;
        assert_eq!(asserts[0].negated, true);
        assert_eq!(asserts[0].target, AssertTarget::Status);
        assert_eq!(
            asserts[0].matcher,
            Some(Matcher::Equals(AssertValue::Number(404.0)))
        );

        assert_eq!(asserts[1].negated, true);
        assert_eq!(
            asserts[1].target,
            AssertTarget::Body(BodyTarget::Path(vec![PathSegment::Key("error".to_string())]))
        );
        assert_eq!(asserts[1].matcher, None);

        assert_eq!(asserts[2].negated, true);
        assert_eq!(
            asserts[2].target,
            AssertTarget::Header("x-deprecated".to_string())
        );
        assert_eq!(asserts[2].matcher, Some(Matcher::Present));

        assert_eq!(asserts[3].negated, true);
        assert_eq!(
            asserts[3].target,
            AssertTarget::Cookie("tracking".to_string(), None)
        );
        assert_eq!(asserts[3].matcher, Some(Matcher::Present));

        assert_eq!(asserts[4].negated, true);
        assert_eq!(
            asserts[4].target,
            AssertTarget::Body(BodyTarget::Path(vec![
                PathSegment::Key("user".to_string()),
                PathSegment::Key("name".to_string()),
            ]))
        );
        assert_eq!(
            asserts[4].matcher,
            Some(Matcher::Equals(AssertValue::String("root".to_string())))
        );
    }

    #[test]
    fn test_parse_assert_headers_cookies() {
        let input = r#"
        group g
            test t
                get "/x"
                assert header.content-type equals "application/json"
                assert header.x-request-id present
                assert cookie.session present
                assert cookie.session equals "abc123"
                assert cookie.session.httponly
                assert cookie.session.secure
                assert cookie.session.samesite equals "Strict"
                assert cookie.tracking absent
            end
        end
        "#;
        let suite = parse(input).unwrap();
        let a = &suite.0[0].tests[0].asserts;

        assert_eq!(
            a[0].target,
            AssertTarget::Header("content-type".to_string())
        );
        assert_eq!(
            a[0].matcher,
            Some(Matcher::Equals(AssertValue::String(
                "application/json".to_string()
            )))
        );

        assert_eq!(a[1].matcher, Some(Matcher::Present));

        assert_eq!(
            a[2].target,
            AssertTarget::Cookie("session".to_string(), None)
        );
        assert_eq!(a[2].matcher, Some(Matcher::Present));

        assert_eq!(
            a[3].matcher,
            Some(Matcher::Equals(AssertValue::String("abc123".to_string())))
        );

        assert_eq!(
            a[4].target,
            AssertTarget::Cookie("session".to_string(), Some("httponly".to_string()))
        );
        assert_eq!(a[4].matcher, None);

        assert_eq!(
            a[6].target,
            AssertTarget::Cookie("session".to_string(), Some("samesite".to_string()))
        );
        assert_eq!(
            a[6].matcher,
            Some(Matcher::Equals(AssertValue::String("Strict".to_string())))
        );

        assert_eq!(
            a[7].target,
            AssertTarget::Cookie("tracking".to_string(), None)
        );
        assert_eq!(a[7].matcher, Some(Matcher::Absent));
    }

    #[test]
    fn test_parse_assert_matchers() {
        let input = r#"
        group g
            test t
                get "/x"
                assert body contains "success"
                assert body equals "ok"
                assert body empty
                assert body.count equals 3
                assert body.score gt 90
                assert body.enabled true
                assert body.items length 5
                assert body.tags length gt 0
                assert duration lt 500
                assert duration lte 1000
                assert body.email matches r"^[\w.]+@[\w.]+\.[a-z]{2,}$"
                assert header.content-type contains "json"
                assert body.user.name starts-with "ali"
                assert body.user.name ends-with "ice"
                assert body.count not-equals 0
            end
        end
        "#;
        let suite = parse(input).unwrap();
        let a = &suite.0[0].tests[0].asserts;

        assert_eq!(a[0].target, AssertTarget::Body(BodyTarget::Raw));
        assert_eq!(
            a[0].matcher,
            Some(Matcher::Contains(AssertValue::String("success".to_string())))
        );

        assert_eq!(
            a[1].matcher,
            Some(Matcher::Equals(AssertValue::String("ok".to_string())))
        );
        assert_eq!(a[2].matcher, Some(Matcher::Empty));

        assert_eq!(
            a[3].matcher,
            Some(Matcher::Equals(AssertValue::Number(3.0)))
        );
        assert_eq!(a[4].matcher, Some(Matcher::Gt(AssertValue::Number(90.0))));
        assert_eq!(
            a[5].matcher,
            Some(Matcher::Equals(AssertValue::Bool(true)))
        );

        assert_eq!(
            a[6].matcher,
            Some(Matcher::Length(LengthArg::Exact(AssertValue::Number(5.0))))
        );
        assert_eq!(
            a[7].matcher,
            Some(Matcher::Length(LengthArg::Gt(AssertValue::Number(0.0))))
        );

        assert_eq!(a[8].target, AssertTarget::Duration);
        assert_eq!(a[8].matcher, Some(Matcher::Lt(AssertValue::Number(500.0))));
        assert_eq!(
            a[9].matcher,
            Some(Matcher::Lte(AssertValue::Number(1000.0)))
        );

        assert_eq!(
            a[10].matcher,
            Some(Matcher::Matches(
                r"^[\w.]+@[\w.]+\.[a-z]{2,}$".to_string()
            ))
        );

        assert_eq!(
            a[11].matcher,
            Some(Matcher::Contains(AssertValue::String("json".to_string())))
        );

        assert_eq!(
            a[12].matcher,
            Some(Matcher::StartsWith(AssertValue::String("ali".to_string())))
        );
        assert_eq!(
            a[13].matcher,
            Some(Matcher::EndsWith(AssertValue::String("ice".to_string())))
        );
        assert_eq!(
            a[14].matcher,
            Some(Matcher::NotEquals(AssertValue::Number(0.0)))
        );
    }

    #[test]
    fn test_parse_path_segments() {
        assert_eq!(
            parse_path("user.name"),
            vec![
                PathSegment::Key("user".to_string()),
                PathSegment::Key("name".to_string()),
            ]
        );
        assert_eq!(
            parse_path("items[0]"),
            vec![
                PathSegment::Key("items".to_string()),
                PathSegment::Index(0),
            ]
        );
        assert_eq!(
            parse_path("[0].id"),
            vec![PathSegment::Index(0), PathSegment::Key("id".to_string()),]
        );
        assert_eq!(
            parse_path("a.b[2].c"),
            vec![
                PathSegment::Key("a".to_string()),
                PathSegment::Key("b".to_string()),
                PathSegment::Index(2),
                PathSegment::Key("c".to_string()),
            ]
        );
    }
}
