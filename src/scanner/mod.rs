use std::iter::Peekable;
use std::str::CharIndices;

use crate::scanner::token::{Token, TokenType};

pub mod token;

pub struct Scanner<'source> {
    source: &'source str,
    chars: Peekable<CharIndices<'source>>,
    /// Starting offset into the source code of the current token
    token_start: usize,
    line: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unterminated string")]
    UnterminatedString,
    #[error("Unexpected charcer {0}")]
    UnexpectedCharacter(char),
}

impl<'source> Iterator for Scanner<'source> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, c) = self.chars.next()?;
        self.token_start = offset;

        let token = match c {
            '(' => self.make_token_len(TokenType::LEFT_PAREN, 1),
            ')' => self.make_token_len(TokenType::RIGHT_PAREN, 1),
            '{' => self.make_token_len(TokenType::LEFT_BRACE, 1),
            '}' => self.make_token_len(TokenType::RIGHT_BRACE, 1),
            '.' => self.make_token_len(TokenType::DOT, 1),
            ',' => self.make_token_len(TokenType::COMMA, 1),
            ';' => self.make_token_len(TokenType::SEMICOLON, 1),
            '+' => self.make_token_len(TokenType::PLUS, 1),
            '-' => self.make_token_len(TokenType::MINUS, 1),
            '*' => self.make_token_len(TokenType::STAR, 1),
            '/' => match self.next_char_eq('/') {
                Some(_) => {
                    self.skip_chars_while(|c| c != '\n');
                    return self.next();
                }
                None => self.make_token_len(TokenType::SLASH, 1),
            },
            '!' => match self.next_char_eq('=') {
                Some(_) => self.make_token_len(TokenType::BANG_EQUAL, 2),
                None => self.make_token_len(TokenType::BANG, 1),
            },
            '=' => match self.next_char_eq('=') {
                Some(_) => self.make_token_len(TokenType::EQUAL_EQUAL, 2),
                None => self.make_token_len(TokenType::EQUAL, 1),
            },
            '>' => match self.next_char_eq('=') {
                Some(_) => self.make_token_len(TokenType::GREATER_EQUAL, 2),
                None => self.make_token_len(TokenType::GREATER, 1),
            },
            '<' => match self.next_char_eq('=') {
                Some(_) => self.make_token_len(TokenType::LESS_EQUAL, 2),
                None => self.make_token_len(TokenType::LESS, 1),
            },
            ' ' | '\r' | '\t' => return self.next(),
            '\n' => {
                self.line += 1;
                return self.next();
            }
            '"' => return Some(self.string()),
            '0'..='9' => self.number(),
            '_' | 'a'..='z' | 'A'..='Z' => self.identifier(),
            _ => return Some(Err(Error::UnexpectedCharacter(c))),
        };
        Some(Ok(token))
    }
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source str) -> Scanner<'source> {
        Self {
            source,
            chars: source.char_indices().peekable(),
            token_start: 0,
            line: 1,
        }
    }

    /// Consumes and returns the next character if it matches `predicate`.
    fn next_char_if(&mut self, predicate: impl FnOnce(char) -> bool) -> Option<(usize, char)> {
        self.chars.next_if(|(_, c)| predicate(*c))
    }

    /// Consumes and returns the next character if it equals `expected`.
    fn next_char_eq(&mut self, expected: char) -> Option<(usize, char)> {
        self.next_char_if(|c| c == expected)
    }

    /// Consumes consecutive characters maching `predicate` and returns the last one consumed.
    fn skip_chars_while(&mut self, predicate: impl Fn(char) -> bool) -> Option<(usize, char)> {
        let mut last: Option<(usize, char)> = None;
        loop {
            let Some(item) = self.next_char_if(|c| predicate(c)) else {
                break last;
            };
            last = Some(item);
        }
    }

    /// Peeks two characters ahead without consuming either character.
    fn peek_next_char(&mut self) -> Option<(usize, char)> {
        let mut clone = self.chars.clone();
        clone.next()?;
        clone.peek().copied()
    }

    /// Creates a token spanning from the current token's starting offset to `end`.
    fn make_token(&self, typ: TokenType, end: usize) -> Token {
        Token::new(self.token_start, end, typ, self.line)
    }

    /// Creates a token whose lexeme has the given byte length.
    fn make_token_len(&self, typ: TokenType, lexeme_len: usize) -> Token {
        self.make_token(typ, self.token_start + lexeme_len)
    }

    fn string(&mut self) -> Result<Token, Error> {
        while let Some((_, c)) = self.next_char_if(|c| c != '"') {
            if c == '\n' {
                self.line += 1;
            }
        }
        let Some((offset, _)) = self.next_char_eq('"') else {
            return Err(Error::UnterminatedString);
        };
        let token = self.make_token(TokenType::STRING, offset + 1);
        Ok(token)
    }

    fn number(&mut self) -> Token {
        let mut last = self.skip_chars_while(|c| c.is_ascii_digit());
        if let Some((_, '.')) = self.chars.peek()
            && let Some((_, n)) = self.peek_next_char()
            && n.is_ascii_digit()
        {
            // comsume the '.'
            self.chars.next();
            last = self.skip_chars_while(|c| c.is_ascii_digit());
        }
        let offset = last.map(|i| i.0).unwrap_or(self.token_start);
        let token = self.make_token(TokenType::NUMBER, offset + 1);
        token
    }

    fn identifier(&mut self) -> Token {
        let mut last = self.skip_chars_while(|c| c == '_' || c.is_ascii_alphanumeric());
        let offset = last.map(|i| i.0).unwrap_or(self.token_start);
        let typ = match &self.source[self.token_start..offset + 1] {
            "and" => TokenType::AND,
            "class" => TokenType::CLASS,
            "else" => TokenType::ELSE,
            "false" => TokenType::FALSE,
            "for" => TokenType::FOR,
            "fun" => TokenType::FUN,
            "if" => TokenType::IF,
            "nil" => TokenType::NIL,
            "or" => TokenType::OR,
            "print" => TokenType::PRINT,
            "return" => TokenType::RETURN,
            "super" => TokenType::SUPER,
            "this" => TokenType::THIS,
            "true" => TokenType::TRUE,
            "var" => TokenType::VAR,
            "while" => TokenType::WHILE,
            _ => TokenType::IDENTIFIER,
        };
        let token = self.make_token(typ, offset + 1);
        token
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Error, Scanner, Token, TokenType};

    #[rstest]
    #[case::empty("")]
    #[case::nonempty("+")]
    fn scanner_next_stays_exhausted(#[case] source: &str) {
        let mut scanner = Scanner::new(source);

        for _ in source.chars() {
            assert!(scanner.next().is_some());
        }

        assert!(scanner.next().is_none());
        assert!(scanner.next().is_none());
    }

    #[test]
    fn scanner_next_returns_unexpected_character_error() {
        let mut scanner = Scanner::new("@");

        assert!(matches!(
            scanner.next(),
            Some(Err(Error::UnexpectedCharacter('@')))
        ));
        assert!(scanner.next().is_none());
    }

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
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(token, Token::new(0, 1, typ, 1));
        assert_eq!(token.lexeme(source), source);
    }

    #[rstest]
    #[case::bang("!", TokenType::BANG)]
    #[case::bang_equal("!=", TokenType::BANG_EQUAL)]
    #[case::equal("=", TokenType::EQUAL)]
    #[case::equal_equal("==", TokenType::EQUAL_EQUAL)]
    #[case::greater(">", TokenType::GREATER)]
    #[case::greater_equal(">=", TokenType::GREATER_EQUAL)]
    #[case::less("<", TokenType::LESS)]
    #[case::less_equal("<=", TokenType::LESS_EQUAL)]
    fn scanner_next_returns_one_or_two_character_token(
        #[case] source: &str,
        #[case] typ: TokenType,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(token, Token::new(0, source.len(), typ, 1));
        assert_eq!(token.lexeme(source), source);
    }

    #[rstest]
    #[case::spaces("  +", 2, 1)]
    #[case::tab("\t+", 1, 1)]
    #[case::carriage_return("\r+", 1, 1)]
    #[case::newline("\n+", 1, 2)]
    #[case::multiple_newlines("\n\n+", 2, 3)]
    #[case::mixed_white_space(" \t\r\n+", 4, 2)]
    #[case::trailing_white_space("+ \t\n", 0, 1)]
    fn scanner_next_skips_white_space(
        #[case] source: &str,
        #[case] start: usize,
        #[case] line: usize,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(token, Token::new(start, start + 1, TokenType::PLUS, line));
        assert_eq!(token.lexeme(source), "+");
    }

    #[test]
    fn scanner_next_skips_white_space_between_tokens() {
        let source = "+ \n-";
        let mut scanner = Scanner::new(source);
        let first = scanner.next().unwrap().unwrap();
        let second = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(first, Token::new(0, 1, TokenType::PLUS, 1));
        assert_eq!(first.lexeme(source), "+");
        assert_eq!(second, Token::new(3, 4, TokenType::MINUS, 2));
        assert_eq!(second.lexeme(source), "-");
    }

    #[test]
    fn scanner_next_returns_none_for_only_white_space() {
        let mut scanner = Scanner::new(" \t\r\n");

        assert!(scanner.next().is_none());
    }

    #[rstest]
    #[case::empty_comment("//\n+", &[Token::new(3, 4, TokenType::PLUS, 2)])]
    #[case::comment_text("//a\n+", &[Token::new(4, 5, TokenType::PLUS, 2)])]
    #[case::operators_in_comment("//+!=\n-", &[Token::new(6, 7, TokenType::MINUS, 2)])]
    #[case::consecutive_comments("//a\n//b\n+", &[Token::new(8, 9, TokenType::PLUS, 3)])]
    #[case::comment_between_tokens("+//text\n-", &[Token::new(0, 1, TokenType::PLUS, 1), Token::new(8, 9, TokenType::MINUS, 2)])]
    #[case::white_space_around_comment(" \t//text\n +", &[Token::new(10, 11, TokenType::PLUS, 2)])]
    #[case::trailing_comment("+//text", &[Token::new(0, 1, TokenType::PLUS, 1)])]
    fn scanner_next_skips_comments(#[case] source: &str, #[case] expected: &[Token]) {
        let mut scanner = Scanner::new(source);

        for &token in expected {
            assert!(matches!(scanner.next(), Some(Ok(actual)) if actual == token));
        }

        assert!(scanner.next().is_none());
    }

    #[rstest]
    #[case::empty_comment("//")]
    #[case::comment_text("//text")]
    fn scanner_next_returns_none_for_comment_at_eof(#[case] source: &str) {
        let mut scanner = Scanner::new(source);

        assert!(scanner.next().is_none());
    }

    #[rstest]
    #[case::empty("\"\"", 0, "\"\"")]
    #[case::text("\"hello\"", 0, "\"hello\"")]
    #[case::white_space_inside("\"hello world\"", 0, "\"hello world\"")]
    #[case::operators_inside("\"+ != // text\"", 0, "\"+ != // text\"")]
    #[case::unicode("\"café 🦀\"", 0, "\"café 🦀\"")]
    #[case::leading_white_space("  \"hello\"", 2, "\"hello\"")]
    fn scanner_next_returns_string(
        #[case] source: &str,
        #[case] start: usize,
        #[case] lexeme: &str,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(token, Token::new(start, source.len(), TokenType::STRING, 1));
        assert_eq!(token.lexeme(source), lexeme);
    }

    #[rstest]
    #[case::one_newline("\"a\nb\"", 0, 5, 2, "\"a\nb\"")]
    #[case::multiple_newlines("\"a\nb\nc\"", 0, 7, 3, "\"a\nb\nc\"")]
    #[case::newline_before("\n\"a\nb\"", 1, 6, 3, "\"a\nb\"")]
    #[case::newline_after("\"a\nb\"\n", 0, 5, 2, "\"a\nb\"")]
    fn scanner_next_tracks_lines_in_string(
        #[case] source: &str,
        #[case] start: usize,
        #[case] end: usize,
        #[case] line: usize,
        #[case] lexeme: &str,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(token, Token::new(start, end, TokenType::STRING, line));
        assert_eq!(token.lexeme(source), lexeme);
    }

    #[rstest]
    #[case::opening_quote("\"")]
    #[case::text("\"hello")]
    #[case::multiline("\"hello\nworld")]
    fn scanner_next_returns_unterminated_string_error(#[case] source: &str) {
        let mut scanner = Scanner::new(source);

        assert!(matches!(
            scanner.next(),
            Some(Err(Error::UnterminatedString))
        ));
        assert!(scanner.next().is_none());
    }

    #[rstest]
    #[case::zero("0", 0, 1, "0")]
    #[case::nine("9", 0, 1, "9")]
    #[case::multiple_digits("1234567890", 0, 1, "1234567890")]
    #[case::leading_zeros("007", 0, 1, "007")]
    #[case::leading_space(" 42", 1, 1, "42")]
    #[case::leading_space_with_single_digit(" 5", 1, 1, "5")]
    #[case::leading_spaces("  42", 2, 1, "42")]
    #[case::leading_newline("\n42", 1, 2, "42")]
    #[case::trailing_space("42 ", 0, 1, "42")]
    #[case::trailing_spaces("42  ", 0, 1, "42")]
    #[case::trailing_newline("42\n", 0, 1, "42")]
    fn scanner_next_returns_number(
        #[case] source: &str,
        #[case] start: usize,
        #[case] line: usize,
        #[case] lexeme: &str,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(
            token,
            Token::new(start, start + lexeme.len(), TokenType::NUMBER, line)
        );
        assert_eq!(token.lexeme(source), lexeme);
    }

    #[rstest]
    #[case::zero("0.0", 0, 1, "0.0")]
    #[case::leading_zeros("00123.45", 0, 1, "00123.45")]
    #[case::leading_space(" 42.5", 1, 1, "42.5")]
    #[case::leading_spaces("  42.5", 2, 1, "42.5")]
    #[case::leading_newline("\n42.5", 1, 2, "42.5")]
    #[case::trailing_zeros("12.3400", 0, 1, "12.3400")]
    #[case::trailing_space("42.5 ", 0, 1, "42.5")]
    #[case::trailing_spaces("42.5  ", 0, 1, "42.5")]
    #[case::trailing_newline("42.5\n", 0, 1, "42.5")]
    fn scanner_next_returns_floating_point_number(
        #[case] source: &str,
        #[case] start: usize,
        #[case] line: usize,
        #[case] lexeme: &str,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(
            token,
            Token::new(start, start + lexeme.len(), TokenType::NUMBER, line)
        );
        assert_eq!(token.lexeme(source), lexeme);
    }

    #[rstest]
    #[case::and("and", TokenType::AND)]
    #[case::class("class", TokenType::CLASS)]
    #[case::else_keyword("else", TokenType::ELSE)]
    #[case::false_keyword("false", TokenType::FALSE)]
    #[case::for_keyword("for", TokenType::FOR)]
    #[case::fun("fun", TokenType::FUN)]
    #[case::if_keyword("if", TokenType::IF)]
    #[case::nil("nil", TokenType::NIL)]
    #[case::or("or", TokenType::OR)]
    #[case::print("print", TokenType::PRINT)]
    #[case::return_keyword("return", TokenType::RETURN)]
    #[case::super_keyword("super", TokenType::SUPER)]
    #[case::this("this", TokenType::THIS)]
    #[case::true_keyword("true", TokenType::TRUE)]
    #[case::var("var", TokenType::VAR)]
    #[case::while_keyword("while", TokenType::WHILE)]
    fn scanner_next_returns_keyword(#[case] source: &str, #[case] typ: TokenType) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(token, Token::new(0, source.len(), typ, 1));
        assert_eq!(token.lexeme(source), source);
    }

    #[rstest]
    #[case::single_letter("a", 0, 1, "a")]
    #[case::lowercase("hello", 0, 1, "hello")]
    #[case::uppercase("HELLO", 0, 1, "HELLO")]
    #[case::mixed_case("helloWorld", 0, 1, "helloWorld")]
    #[case::with_digits("hello123", 0, 1, "hello123")]
    #[case::underscore("_", 0, 1, "_")]
    #[case::leading_underscore("_hello", 0, 1, "_hello")]
    #[case::internal_underscore("hello_world", 0, 1, "hello_world")]
    #[case::trailing_underscore("hello_", 0, 1, "hello_")]
    #[case::keyword_prefix("className", 0, 1, "className")]
    #[case::uppercase_keyword("CLASS", 0, 1, "CLASS")]
    #[case::leading_space(" hello", 1, 1, "hello")]
    #[case::leading_spaces("  hello", 2, 1, "hello")]
    #[case::leading_newline("\nhello", 1, 2, "hello")]
    #[case::trailing_space("hello ", 0, 1, "hello")]
    #[case::trailing_spaces("hello  ", 0, 1, "hello")]
    #[case::trailing_newline("hello\n", 0, 1, "hello")]
    fn scanner_next_returns_identifier(
        #[case] source: &str,
        #[case] start: usize,
        #[case] line: usize,
        #[case] lexeme: &str,
    ) {
        let mut scanner = Scanner::new(source);
        let token = scanner.next().unwrap().unwrap();
        assert!(scanner.next().is_none());

        assert_eq!(
            token,
            Token::new(start, start + lexeme.len(), TokenType::IDENTIFIER, line)
        );
        assert_eq!(token.lexeme(source), lexeme);
    }
}
