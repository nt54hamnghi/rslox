use std::iter::Peekable;
use std::str::Chars;

use crate::error::Report;
use crate::scanner::token::{Literal, Token, TokenType};

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
                '(' => self.make_token(TokenType::LeftParen, c),
                ')' => self.make_token(TokenType::RightParen, c),
                '{' => self.make_token(TokenType::LeftBrace, c),
                '}' => self.make_token(TokenType::RightBrace, c),
                '*' => self.make_token(TokenType::Star, c),
                '.' => self.make_token(TokenType::Dot, c),
                ',' => self.make_token(TokenType::Comma, c),
                '+' => self.make_token(TokenType::Plus, c),
                '-' => self.make_token(TokenType::Minus, c),
                ';' => self.make_token(TokenType::Semicolon, c),
                '=' => match self.next_match('=') {
                    Some(nc) => self.make_token_from(TokenType::EqualEqual, [c, nc]),
                    None => self.make_token(TokenType::Equal, c),
                },
                '!' => match self.next_match('=') {
                    Some(nc) => self.make_token_from(TokenType::BangEqual, [c, nc]),
                    None => self.make_token(TokenType::Bang, c),
                },
                '<' => match self.next_match('=') {
                    Some(nc) => self.make_token_from(TokenType::LessEqual, [c, nc]),
                    None => self.make_token(TokenType::Less, c),
                },
                '>' => match self.next_match('=') {
                    Some(nc) => self.make_token_from(TokenType::GreaterEqual, [c, nc]),
                    None => self.make_token(TokenType::Greater, c),
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

    /// Creates a token at the current line with no literal value.
    fn make_token(&self, typ: TokenType, lexeme: impl Into<String>) -> Token {
        Token::new(typ, lexeme.into(), None, self.line)
    }

    /// Creates a token from items that can be collected into a String.
    /// Useful for building lexemes from character iterators or arrays.
    fn make_token_from<T, I>(&self, typ: TokenType, value: I) -> Token
    where
        I: IntoIterator<Item = T>,
        String: FromIterator<T>,
    {
        let lexeme = value.into_iter().collect::<String>();
        self.make_token(typ, lexeme)
    }

    /// Creates a token with an associated literal value (e.g., the numeric value for NUMBER tokens,
    /// the string content for STRING tokens).
    fn make_literal_token(
        &self,
        typ: TokenType,
        lexeme: impl Into<String>,
        literal: Literal,
    ) -> Token {
        Token::new(typ, lexeme.into(), Some(literal), self.line)
    }

    /// Creates a token with a literal value from items that can be collected into a String.
    /// Combines `make_token_from` with literal value support.
    fn make_literal_token_from<T, I>(&self, typ: TokenType, value: I, literal: Literal) -> Token
    where
        I: IntoIterator<Item = T>,
        String: FromIterator<T>,
    {
        let lexeme = value.into_iter().collect::<String>();
        self.make_literal_token(typ, lexeme, literal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(">=", vec![
        "GREATER_EQUAL >= null",
        "EOF  null",
    ])]
    #[case("<<<=>>>=", vec![
        "LESS < null",
        "LESS < null",
        "LESS_EQUAL <= null",
        "GREATER > null",
        "GREATER > null",
        "GREATER_EQUAL >= null",
        "EOF  null",
    ])]
    #[case("<=>>=>>=", vec![
        "LESS_EQUAL <= null",
        "GREATER > null",
        "GREATER_EQUAL >= null",
        "GREATER > null",
        "GREATER_EQUAL >= null",
        "EOF  null",
    ])]
    #[case("(){===!}", vec![
        "LEFT_PAREN ( null",
        "RIGHT_PAREN ) null",
        "LEFT_BRACE { null",
        "EQUAL_EQUAL == null",
        "EQUAL = null",
        "BANG ! null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("!=", vec![
        "BANG_EQUAL != null",
        "EOF  null",
    ])]
    #[case("!!===", vec![
        "BANG ! null",
        "BANG_EQUAL != null",
        "EQUAL_EQUAL == null",
        "EOF  null",
    ])]
    #[case("!{!}(!===)=", vec![
        "BANG ! null",
        "LEFT_BRACE { null",
        "BANG ! null",
        "RIGHT_BRACE } null",
        "LEFT_PAREN ( null",
        "BANG_EQUAL != null",
        "EQUAL_EQUAL == null",
        "RIGHT_PAREN ) null",
        "EQUAL = null",
        "EOF  null",
    ])]
    #[case("{(!==$=#)}", vec![
        "LEFT_BRACE { null",
        "LEFT_PAREN ( null",
        "BANG_EQUAL != null",
        "EQUAL = null",
        "[line 1] Error: Unexpected character: $",
        "EQUAL = null",
        "[line 1] Error: Unexpected character: #",
        "RIGHT_PAREN ) null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
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
