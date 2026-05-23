use std::iter::Peekable;

use super::{
    ast::*,
    helpers::parse_target,
    lexer::Token,
};

pub(super) struct Parser<I: Iterator<Item = Token>>(Peekable<I>);

impl<I: Iterator<Item = Token>> Parser<I> {
    pub(super) fn new(input: I) -> Self {
        Parser(input.peekable())
    }

    pub(super) fn parse(mut self) -> Result<Suite, String> {
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
