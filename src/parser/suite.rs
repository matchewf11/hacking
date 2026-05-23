use std::{
    str::Bytes,
    iter::Peekable,
};

#[derive(Debug, PartialEq)]
enum Token {
    Ident(String),
    End,
    Group,
    Test,
    Assert,
}

impl Token {
    fn from_string(s: String) -> Self {
        match s.as_str() {
            "end" => Token::End,
            "group" => Token::Group,
            "test" => Token::Test,
            "assert" => Token::Assert,
            _ => Token::Ident(s),
        }
    }
}

struct Lexer<'a>(Peekable<Bytes<'a>>);

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer(input.bytes().peekable())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.0.peek() {
            None => return None,
            Some(b) if b.is_ascii_whitespace() => {
                self.0.next();
                return self.next();
            }
            _ => {
                self.0.by_ref().take_while(|b: &u8| !b.is_ascii_whitespace()).collect::<Vec<_>>()
            }
        };


        if res.is_empty() {
            None
        } else {
            Some(Token::from_string(std::str::from_utf8(&res).unwrap().to_string()))
        }
    }
}


#[derive(PartialEq, Debug)]
pub struct Suite(Vec<Group>);

#[derive(PartialEq, Debug)]
pub struct Group {
    name: String,
    tests: Vec<Test>,
}

#[derive(PartialEq, Debug)]
pub struct Test {
    name: String,
    value: String,
    asserts: Vec<String>,
}

struct Parser<I: Iterator<Item = Token>>(Peekable<I>);

impl<I: Iterator<Item = Token>> Parser<I> {
    fn new(input: I) -> Self {
        Parser(input.peekable())
    }

    fn parse(mut self) -> Result<Suite, String> {
        let mut groups = Vec::new();
        while let Some(g) = self.parse_group()? {
            groups.push(g);
        }
        Ok(Suite(groups))
    }

    fn parse_group(&mut self) -> Result<Option<Group>, String> {
        match self.0.peek() {
            None => return Ok(None),
            Some(Token::Group) => {
                self.0.next();
            }
            _ => return Err("wanted group".to_string()),
        }

        let name = match self.0.next() {
            Some(Token::Ident(s)) => s,
            _ => return Err("wanted group name".to_string()),
        };

        let mut tests = Vec::new();

        loop {
            match self.0.peek() {
                Some(Token::Test) => {
                    tests.push(self.parse_test()?);
                }
                Some(Token::End) => {
                    self.0.next();
                    break;
                }
                Some(tok) => {
                    return Err(format!("unexpected token in group: {:?}", tok));
                }
                None => return Err("unexpected eof in group".to_string()),
            }
        }

        Ok(Some(Group { name, tests }))
    }

    fn parse_test(&mut self) -> Result<Test, String> {
        match self.0.next() {
            Some(Token::Test) => {}
            _ => return Err("wanted test".to_string()),
        }

        let name = match self.0.next() {
            Some(Token::Ident(s)) => s,
            _ => return Err("wanted test name".to_string()),
        };

        let _method = match self.0.next() {
            Some(Token::Ident(s)) => s,
            _ => return Err("wanted method".to_string()),
        };

        let value = match self.0.next() {
            Some(Token::Ident(s)) => s,
            _ => return Err("wanted value".to_string()),
        };

        let mut asserts = Vec::new();

        loop {
            match self.0.peek() {
                Some(Token::Assert) => {
                    self.0.next();

                    let first = match self.0.next() {
                        Some(Token::Ident(s)) => s,
                        _ => return Err("wanted assert value".to_string()),
                    };

                    let assert_value = match self.0.peek() {
                        Some(Token::Ident(_)) => {
                            match self.0.next() {
                                Some(Token::Ident(s)) => {
                                    format!("{first} {s}")
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => first,
                    };

                    asserts.push(assert_value);
                }

                Some(Token::End) => {
                    self.0.next();
                    break;
                }

                Some(tok) => {
                    return Err(format!("unexpected token in test: {:?}", tok));
                }

                None => return Err("unexpected eof in test".to_string()),
            }
        }

        Ok(Test {
            name,
            value,
            asserts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let lexer = Lexer::new(input);
        let parser = Parser::new(lexer);

        let suite = parser.parse().unwrap();

        assert_eq!(
            suite,
            Suite(vec![
                Group {
                    name: "auth".to_string(),
                    tests: vec![
                        Test {
                            name: "abc".to_string(),
                            value: r#""/foo""#.to_string(),
                            asserts: vec![
                                "status 200".to_string(),
                                "body.foo".to_string(),
                            ],
                        }
                    ],
                },
                Group {
                    name: "foo".to_string(),
                    tests: vec![
                        Test {
                            name: "abc".to_string(),
                            value: r#""/foo""#.to_string(),
                            asserts: vec![
                                "status 200".to_string(),
                                "body.foo".to_string(),
                            ],
                        },
                        Test {
                            name: "ab".to_string(),
                            value: r#""/foo""#.to_string(),
                            asserts: vec![
                                "status 200".to_string(),
                                r#"body.foo "bar""#.to_string(),
                            ],
                        },
                    ],
                },
            ])
        );
    }

    #[test]
    fn test_lexer() {
        let input = r#"
        group auth
            test abc
                get "/foo"
                assert status 200
            end
        end
        "#;

        let lexer = Lexer::new(input).collect::<Vec<_>>();
        assert_eq!(
            lexer,
            vec![
                Token::Group,
                Token::Ident("auth".to_string()),
                Token::Test,
                Token::Ident("abc".to_string()),
                Token::Ident("get".to_string()),
                Token::Ident("\"/foo\"".to_string()),
                Token::Assert,
                Token::Ident("status".to_string()),
                Token::Ident("200".to_string()),
                Token::End,
                Token::End,
            ],
        );
    }
}

