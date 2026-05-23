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
                let mut res = Vec::new();

                while let Some(b) = self.0.peek() && !b.is_ascii_whitespace() {
                    res.push(self.0.next().unwrap());
                }

                res
            }
        };


        if res.is_empty() {
            None
        } else {
            Some(Token::from_string(std::str::from_utf8(&res).unwrap().to_string()))
        }
    }
}

pub fn parse(input: &str) -> Result<Suite, String> {
    Parser::new(Lexer::new(input).collect::<Vec<_>>().into_iter()).parse()
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

        while let Some(group) = self.parse_group()? {
            groups.push(group);
        }

        Ok(Suite(groups))
    }

    fn expect_ident(&mut self, msg: &str) -> Result<String, String> {
        match self.0.next() {
            Some(Token::Ident(s)) => Ok(s),
            other => Err(format!("{msg}, got {:?}", other)),
        }
    }

    fn parse_group(&mut self) -> Result<Option<Group>, String> {

        match self.0.peek() {
            None => return Ok(None),
            Some(Token::Group) => {
                self.0.next();
            }
            other => {
                return Err(format!("expected `group`, got {:?}", other));
            }
        }

        let name = self.expect_ident("expected group name")?;

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
                None => {
                    return Err("unexpected EOF in group".to_string());
                }
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

        let mut value_parts = Vec::new();
        let mut asserts = Vec::new();

        loop {
            match self.0.peek() {
                Some(Token::Ident(_)) => {
                    if let Some(Token::Ident(s)) = self.0.next() {
                        value_parts.push(s);
                    }
                }
                Some(Token::Assert) | Some(Token::End) => break,
                Some(tok) => {
                    return Err(format!("unexpected token in test value: {:?}", tok));
                }
                None => return Err("unexpected EOF in test value".to_string()),
            }
        }

        loop {
            match self.0.peek() {
                Some(Token::Assert) => {
                   self.0.next();
                    let mut parts = Vec::new();
                    loop {
                        match self.0.peek() {
                            Some(Token::Ident(_)) => {
                                if let Some(Token::Ident(s)) = self.0.next() {
                                    parts.push(s);
                                }
                            }
                            Some(Token::Assert | Token::End) => break,
                            Some(tok) => {
                                return Err(format!("unexpected token in assert: {:?}", tok));
                            }
                            None => return Err("unexpected EOF in assert".to_string()),
                        }
                    }
                    asserts.push(parts.join(" "));
                }
                Some(Token::End) => {
                    self.0.next();
                    break;
                }
                Some(tok) => {
                    return Err(format!("unexpected token in test asserts: {:?}", tok));
                }
                None => return Err("unexpected EOF in test asserts".to_string()),
            }
        }

        Ok(Test {
            name,
            value: value_parts.join(" "),
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
                            value: r#"get "/foo""#.to_string(),
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
                            value: r#"get "/foo""#.to_string(),
                            asserts: vec![
                                "status 200".to_string(),
                                "body.foo".to_string(),
                            ],
                        },
                        Test {
                            name: "ab".to_string(),
                            value: r#"get "/foo""#.to_string(),
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

