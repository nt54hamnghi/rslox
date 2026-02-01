use std::str::Chars;

use crate::error::Report;
use crate::scanner::token::{Token, TokenType};

pub mod token;

pub struct Scanner<'src> {
    // Raw source code
    _raw: &'src str,
    line: u32,
    chars: Chars<'src>,
    at_end: bool,
}

impl<'src> Scanner<'src> {
    pub fn scan_tokens(source: &'src str) -> Scanner<'src> {
        Self {
            _raw: source,
            line: 1,
            chars: source.chars(),
            at_end: false,
        }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token, Report>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            return None;
        }

        let token = match self.chars.next() {
            Some(c) => match c {
                '(' => Token::new(TokenType::LeftParen, c, None, self.line),
                ')' => Token::new(TokenType::RightParen, c, None, self.line),
                '{' => Token::new(TokenType::LeftBrace, c, None, self.line),
                '}' => Token::new(TokenType::RightBrace, c, None, self.line),
                '*' => Token::new(TokenType::Star, c, None, self.line),
                '.' => Token::new(TokenType::Dot, c, None, self.line),
                ',' => Token::new(TokenType::Comma, c, None, self.line),
                '+' => Token::new(TokenType::Plus, c, None, self.line),
                '-' => Token::new(TokenType::Minus, c, None, self.line),
                ';' => Token::new(TokenType::Semicolon, c, None, self.line),
                _ => {
                    return Some(Err(Report::error(
                        self.line,
                        format!("Unexpected character: {c}"),
                    )));
                }
            },
            None => {
                self.at_end = true;
                Token::new_eof(self.line)
            }
        };

        Some(Ok(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("(", vec![
        "LEFT_PAREN ( null",
        "EOF  null",
    ])]
    #[case("))", vec![
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("())))", vec![
        "LEFT_PAREN ( null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("((()())", vec![
        "LEFT_PAREN ( null",
        "LEFT_PAREN ( null",
        "LEFT_PAREN ( null",
        "RIGHT_PAREN ) null",
        "LEFT_PAREN ( null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("}", vec![
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("{{}}", vec![
        "LEFT_BRACE { null",
        "LEFT_BRACE { null",
        "RIGHT_BRACE } null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("}{}}", vec![
        "RIGHT_BRACE } null",
        "LEFT_BRACE { null",
        "RIGHT_BRACE } null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("{{)})}(", vec![
        "LEFT_BRACE { null",
        "LEFT_BRACE { null",
        "RIGHT_PAREN ) null",
        "RIGHT_BRACE } null",
        "RIGHT_PAREN ) null",
        "RIGHT_BRACE } null",
        "LEFT_PAREN ( null",
        "EOF  null",
    ])]
    #[case("+-", vec![
        "PLUS + null",
        "MINUS - null",
        "EOF  null",
    ])]
    #[case("++--**..,,;;", vec![
        "PLUS + null",
        "PLUS + null",
        "MINUS - null",
        "MINUS - null",
        "STAR * null",
        "STAR * null",
        "DOT . null",
        "DOT . null",
        "COMMA , null",
        "COMMA , null",
        "SEMICOLON ; null",
        "SEMICOLON ; null",
        "EOF  null",
    ])]
    #[case("-+;.*;,", vec![
        "MINUS - null",
        "PLUS + null",
        "SEMICOLON ; null",
        "DOT . null",
        "STAR * null",
        "SEMICOLON ; null",
        "COMMA , null",
        "EOF  null",
    ])]
    #[case("({*-+;.})", vec![
        "LEFT_PAREN ( null",
        "LEFT_BRACE { null",
        "STAR * null",
        "MINUS - null",
        "PLUS + null",
        "SEMICOLON ; null",
        "DOT . null",
        "RIGHT_BRACE } null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    fn test_scan_parentheses_and_braces(#[case] input: &str, #[case] expected_output: Vec<&str>) {
        let output = Scanner::scan_tokens(input)
            .map(|t| match t {
                Ok(t) => t.to_string(),
                Err(e) => e.to_string(),
            })
            .collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }
}
