use std::{iter::Peekable, str::Bytes};

#[derive(Debug, PartialEq, Clone)]
pub(super) enum Token {
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

pub(super) struct Lexer<'a>(Peekable<Bytes<'a>>);

impl<'a> Lexer<'a> {
    pub(super) fn new(input: &'a str) -> Self {
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
