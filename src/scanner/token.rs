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
    pub(super) fn new(start: usize, end: usize, typ: TokenType, line: usize) -> Self {
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

    // Signals
    EOF,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Token, TokenType};

    #[rstest]
    #[case::whole_source("hello", 0, 5, "hello")]
    #[case::first_character("(hello)", 0, 1, "(")]
    #[case::middle_of_source("var name = 42;", 4, 8, "name")]
    #[case::end_of_source("var name", 4, 8, "name")]
    #[case::multibyte_lexeme("\"café 🦀\";", 1, 11, "café 🦀")]
    fn token_lexeme_returns_source_slice(
        #[case] source: &str,
        #[case] start: usize,
        #[case] end: usize,
        #[case] expected: &str,
    ) {
        let token = Token::new(start, end, TokenType::STRING, 1);

        assert_eq!(token.lexeme(source), expected);
    }
}
