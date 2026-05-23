mod ast;
mod helpers;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

pub use ast::*;

pub fn parse(input: &str) -> Result<Suite, String> {
    parser::Parser::new(lexer::Lexer::new(input).collect::<Vec<_>>().into_iter()).parse()
}
