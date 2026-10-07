use std::iter::Peekable;
use std::str::CharIndices;

pub struct Scanner<'source> {
    chars: Peekable<CharIndices<'source>>,
    /// Starting offset into the source code of the current token
    current_token_start: usize,
    line: usize,
}

impl<'source> Iterator for Scanner<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, c) = self.chars.next()?;
        self.current_token_start = offset;

        let token = match c {
            '(' => self.make_token(TokenType::LEFT_PAREN, 1),
            ')' => self.make_token(TokenType::RIGHT_PAREN, 1),
            '{' => self.make_token(TokenType::LEFT_BRACE, 1),
            '}' => self.make_token(TokenType::RIGHT_BRACE, 1),
            '.' => self.make_token(TokenType::DOT, 1),
            ',' => self.make_token(TokenType::COMMA, 1),
            ';' => self.make_token(TokenType::SEMICOLON, 1),
            '+' => self.make_token(TokenType::PLUS, 1),
            '-' => self.make_token(TokenType::MINUS, 1),
            '*' => self.make_token(TokenType::STAR, 1),
            '/' => self.make_token(TokenType::SLASH, 1),
            '!' => match self.chars.next_if(|(_, c)| *c == '=') {
                Some(_) => self.make_token(TokenType::BANG_EQUAL, 2),
                None => self.make_token(TokenType::BANG, 1),
            },
            '=' => match self.chars.next_if(|(_, c)| *c == '=') {
                Some(_) => self.make_token(TokenType::EQUAL_EQUAL, 2),
                None => self.make_token(TokenType::EQUAL, 1),
            },
            '>' => match self.chars.next_if(|(_, c)| *c == '=') {
                Some(_) => self.make_token(TokenType::GREATER_EQUAL, 2),
                None => self.make_token(TokenType::GREATER, 1),
            },
            '<' => match self.chars.next_if(|(_, c)| *c == '=') {
                Some(_) => self.make_token(TokenType::LESS_EQUAL, 2),
                None => self.make_token(TokenType::LESS, 1),
            },
            _ => todo!(),
        };
        Some(token)
    }
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source str) -> Scanner<'source> {
        Self {
            chars: source.char_indices().peekable(),
            current_token_start: 0,
            line: 1,
        }
    }

    fn make_token(&self, typ: TokenType, lexeme_length: usize) -> Token {
        Token::new(
            self.current_token_start,
            self.current_token_start + lexeme_length,
            typ,
            self.line,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// Starting byte offset of the lexeme in the source code, inclusive
    start: usize,
    /// Ending byte offset of the lexeme in the source code, exclusive
    end: usize,
    typ: TokenType,
    line: usize,
}

impl Token {
    fn new(start: usize, end: usize, typ: TokenType, line: usize) -> Self {
        debug_assert!(start < end);
        Self {
            start,
            end,
            typ,
            line,
        }
    }
    pub fn lexeme<'s>(&self, source: &'s str) -> &'s str {
        &source[self.start..self.end]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    ERROR,
    EOF,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Scanner, Token, TokenType};

    #[rstest]
    #[case::left_paren("(", TokenType::LEFT_PAREN)]
    #[case::right_paren(")", TokenType::RIGHT_PAREN)]
    #[case::left_brace("{", TokenType::LEFT_BRACE)]
    #[case::right_brace("}", TokenType::RIGHT_BRACE)]
    #[case::dot(".", TokenType::DOT)]
    #[case::comma(",", TokenType::COMMA)]
    #[case::semicolon(";", TokenType::SEMICOLON)]
    #[case::plus("+", TokenType::PLUS)]
    #[case::minus("-", TokenType::MINUS)]
    #[case::star("*", TokenType::STAR)]
    #[case::slash("/", TokenType::SLASH)]
    fn scanner_next_returns_single_character_token(#[case] source: &str, #[case] typ: TokenType) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap();
        assert_eq!(scanner.next(), None);

        assert_eq!(token, Token::new(0, 1, typ, 1));
        assert_eq!(token.lexeme(source), source);
    }

    #[rstest]
    #[case::empty("")]
    #[case::nonempty("+")]
    fn scanner_next_stays_exhausted(#[case] source: &str) {
        let mut scanner = Scanner::new(source);

        for _ in source.chars() {
            assert!(scanner.next().is_some());
        }

        assert_eq!(scanner.next(), None);
        assert_eq!(scanner.next(), None);
    }

    #[rstest]
    #[case::whole_source("hello", 0, 5, "hello")]
    #[case::first_character("(hello)", 0, 1, "(")]
    #[case::middle_of_source("var name = 42;", 4, 8, "name")]
    #[case::end_of_source("var name", 4, 8, "name")]
    #[case::empty_source("", 0, 0, "")]
    #[case::multibyte_lexeme("\"café 🦀\";", 1, 11, "café 🦀")]
    fn token_lexeme_returns_source_slice(
        #[case] source: &str,
        #[case] start: usize,
        #[case] end: usize,
        #[case] expected: &str,
    ) {
        let token = Token {
            start,
            end,
            typ: TokenType::STRING,
            line: 1,
        };

        assert_eq!(token.lexeme(source), expected);
    }
}
