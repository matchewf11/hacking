use super::{
    ast::*,
    helpers::parse_path,
    lexer::{Lexer, Token},
    parse,
};

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

    assert_eq!(a[2].target, AssertTarget::Cookie("session".to_string(), None));
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
