use std::str::Chars;

use crate::scanner::token::{Token, TokenType};

pub mod token;

pub struct Scanner<'src> {
    source: &'src str,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Scanner<'src> {
        Self { source }
    }

    pub fn scan_tokens(&self) -> Tokens<'_> {
        Tokens {
            chars: self.source.chars(),
            at_end: false,
        }
    }
}

pub struct Tokens<'src> {
    chars: Chars<'src>,
    at_end: bool,
}

impl<'src> Iterator for Tokens<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            return None;
        }

        let token = match self.chars.next() {
            Some(c) => match c {
                '(' => Token::new(TokenType::LeftParen, '(', None),
                ')' => Token::new(TokenType::RightParen, ')', None),
                '{' => Token::new(TokenType::LeftBrace, '{', None),
                '}' => Token::new(TokenType::RightBrace, '}', None),
                _ => unimplemented!("{c:#?}"),
            },
            None => {
                self.at_end = true;
                Token::eof_token()
            }
        };

        Some(token)
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
    fn test_scan_parentheses(#[case] input: &str, #[case] expected_output: Vec<&str>) {
        let scanner = Scanner::new(input);
        let output = scanner
            .scan_tokens()
            .map(|t| t.to_string())
            .collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }
}
