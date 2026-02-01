use std::iter::Peekable;
use std::str::Chars;

use crate::error::Report;
use crate::scanner::token::{Token, TokenType};

pub mod token;

pub struct Scanner<'src> {
    // Raw source code
    source: &'src str,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Scanner<'src> {
        Self { source }
    }

    pub fn scan_tokens(&self) -> TokenStream<'src> {
        TokenStream {
            line: 1,
            chars: self.source.chars().peekable(),
            at_end: false,
        }
    }
}

pub struct TokenStream<'src> {
    chars: Peekable<Chars<'src>>,
    line: u32,
    at_end: bool,
}

impl<'src> Iterator for TokenStream<'src> {
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
                '=' => match self.next_match('=') {
                    Some(nc) => {
                        let lexeme = [c, nc].iter().collect::<String>();
                        Token::new(TokenType::EqualEqual, lexeme, None, self.line)
                    }

                    None => Token::new(TokenType::Equal, c, None, self.line),
                },
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

impl<'src> TokenStream<'src> {
    /// Attempts to consume the next character if it matches the expected character.
    ///
    /// This method peeks at the next character in the iterator. If it matches the `expected`
    /// character, the character is consumed and returned. Otherwise, `None` is returned and
    /// the iterator position remains unchanged.
    fn next_match(&mut self, expected: char) -> Option<char> {
        let value = self.chars.peek()?;
        if *value == expected {
            return self.chars.next();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("=", vec![
        "EQUAL = null",
        "EOF  null",
    ])]
    #[case("==", vec![
        "EQUAL_EQUAL == null",
        "EOF  null",
    ])]
    #[case("({=}){==}", vec![
        "LEFT_PAREN ( null",
        "LEFT_BRACE { null",
        "EQUAL = null",
        "RIGHT_BRACE } null",
        "RIGHT_PAREN ) null",
        "LEFT_BRACE { null",
        "EQUAL_EQUAL == null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("((#$%===))", vec![
        "LEFT_PAREN ( null",
        "LEFT_PAREN ( null",
        "[line 1] Error: Unexpected character: #",
        "[line 1] Error: Unexpected character: $",
        "[line 1] Error: Unexpected character: %",
        "EQUAL_EQUAL == null",
        "EQUAL = null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("@", vec![
        "[line 1] Error: Unexpected character: @",
        "EOF  null",
    ])]
    #[case(",.$(#", vec![
        "COMMA , null",
        "DOT . null",
        "[line 1] Error: Unexpected character: $",
        "LEFT_PAREN ( null",
        "[line 1] Error: Unexpected character: #",
        "EOF  null",
    ])]
    #[case("%#$@@", vec![
        "[line 1] Error: Unexpected character: %",
        "[line 1] Error: Unexpected character: #",
        "[line 1] Error: Unexpected character: $",
        "[line 1] Error: Unexpected character: @",
        "[line 1] Error: Unexpected character: @",
        "EOF  null",
    ])]
    #[case("{(.#;*@+%)}", vec![
        "LEFT_BRACE { null",
        "LEFT_PAREN ( null",
        "DOT . null",
        "[line 1] Error: Unexpected character: #",
        "SEMICOLON ; null",
        "STAR * null",
        "[line 1] Error: Unexpected character: @",
        "PLUS + null",
        "[line 1] Error: Unexpected character: %",
        "RIGHT_PAREN ) null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
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
    fn test_scanner(#[case] input: &str, #[case] expected_output: Vec<&str>) {
        let scanner = Scanner::new(input);
        let output = scanner
            .scan_tokens()
            .map(|t| match t {
                Ok(t) => t.to_string(),
                Err(e) => e.to_string(),
            })
            .collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }
}
