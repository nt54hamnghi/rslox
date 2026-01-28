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
