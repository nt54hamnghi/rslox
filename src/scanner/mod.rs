use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use std::sync::LazyLock;

use crate::error::Report;
use crate::scanner::token::{Literal, Token, TokenType};

pub mod token;

pub static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fun", TokenType::Fun),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ])
});

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
            lead: None,
            at_end: false,
        }
    }
}

pub struct TokenStream<'src> {
    /// The character iterator for the source code being scanned
    chars: Peekable<Chars<'src>>,
    /// The leading character for multi-character tokens
    lead: Option<char>,
    /// The current line number in the source code
    line: u32,
    /// Whether the end of the token stream has been reached
    at_end: bool,
}

#[derive(Debug)]
pub enum ScanResult {
    Result(Result<Token, Report>),
    Ignore,
}

impl ScanResult {
    fn ok(token: Token) -> ScanResult {
        Self::Result(Ok(token))
    }

    fn err(error: Report) -> ScanResult {
        Self::Result(Err(error))
    }
}

impl<'src> Iterator for TokenStream<'src> {
    type Item = ScanResult;

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
                '/' => match self.next_match('/') {
                    Some(_) => {
                        loop {
                            let Some(_) = self.chars.next_if(|c| *c != '\n') else {
                                break;
                            };
                        }
                        return Some(ScanResult::Ignore);
                    }
                    None => self.make_token(TokenType::Slash, c),
                },
                ' ' | '\t' | '\r' => return Some(ScanResult::Ignore),
                '\n' => {
                    self.line += 1;
                    return Some(ScanResult::Ignore);
                }
                '"' => {
                    self.lead = Some(c);
                    return Some(self.string());
                }
                '0'..='9' => {
                    self.lead = Some(c);
                    return Some(self.number());
                }
                '_' | 'a'..='z' | 'A'..='Z' => {
                    self.lead = Some(c);
                    return Some(self.identifier());
                }
                _ => {
                    let report = Report::error(self.line, format!("Unexpected character: {c}"));
                    return Some(ScanResult::err(report));
                }
            },
            None => {
                self.at_end = true;
                Token::new_eof(self.line)
            }
        };

        Some(ScanResult::ok(token))
    }
}

impl<'src> TokenStream<'src> {
    /// Consume and return the next item if it is equal to expected.
    fn next_match(&mut self, expected: char) -> Option<char> {
        self.chars.next_if_eq(&expected)
    }

    /// Peeks at the character after the next one in the stream without consuming any characters.
    /// This method looks ahead two positions in the character stream.
    fn peek_next(&self) -> Option<char> {
        let mut cloned = self.chars.clone();
        cloned.next()?;
        return cloned.peek().cloned();
    }

    /// Scan an identifier
    fn identifier(&mut self) -> ScanResult {
        let lead = self.lead.take().expect("Expected a leading character");
        let mut lexeme = String::from(lead);

        while let Some(current) = self
            .chars
            .next_if(|c| *c == '_' || c.is_ascii_alphanumeric())
        {
            lexeme.push(current);
        }

        let typ = KEYWORDS
            .get(lexeme.as_str())
            .cloned()
            .unwrap_or(TokenType::Identifier);
        let token = self.make_token(typ, lexeme);

        ScanResult::ok(token)
    }

    /// Scan a number token
    fn number(&mut self) -> ScanResult {
        let lead = self.lead.take().expect("Expected a leading digit");
        let mut lexeme = String::from(lead);

        while let Some(current) = self.chars.next_if(char::is_ascii_digit) {
            lexeme.push(current);
        }

        if let Some('.') = self.chars.peek()
            && let Some(n) = self.peek_next()
            && n.is_ascii_digit()
        {
            // unwrap is safe since peek returned Some('.')
            lexeme.push(self.chars.next().unwrap());
            while let Some(current) = self.chars.next_if(char::is_ascii_alphanumeric) {
                lexeme.push(current);
            }
        };

        let number = lexeme
            .parse::<f64>()
            .expect("Expected a valid double-precision float");
        let token = self.make_literal_token(TokenType::Number, lexeme, number.into());

        ScanResult::ok(token)
    }

    /// Scan a string token
    fn string(&mut self) -> ScanResult {
        let lead = self.lead.take().expect("Expected an opening quote");
        let mut lexeme = String::from(lead);

        while let Some(current) = self.chars.next_if(|c| *c != '"') {
            if current == '\n' {
                self.line += 1;
            }
            lexeme.push(current);
        }

        // reached the end of the input without finding a closing quote
        if self.chars.peek().is_none() {
            let report = Report::error(self.line, "Unterminated string.".into());
            return ScanResult::err(report);
        } else {
            // consume the closing quote
            // unwrap is safe since peek returned Some(_)
            lexeme.push(self.chars.next().unwrap());
        }

        let literal = Literal::from(&lexeme[1..lexeme.len() - 1]);
        let token = self.make_literal_token(TokenType::String, lexeme, literal);

        ScanResult::ok(token)
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
    #[allow(unused)]
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
    #[case("return", vec![
        "RETURN return null",
        "EOF  null",
    ])]
    #[case("AND FOR SUPER while this true FUN false THIS and NIL FALSE or else nil if print WHILE fun class RETURN IF return OR super TRUE ELSE for CLASS PRINT var VAR", vec![
        "IDENTIFIER AND null",
        "IDENTIFIER FOR null",
        "IDENTIFIER SUPER null",
        "WHILE while null",
        "THIS this null",
        "TRUE true null",
        "IDENTIFIER FUN null",
        "FALSE false null",
        "IDENTIFIER THIS null",
        "AND and null",
        "IDENTIFIER NIL null",
        "IDENTIFIER FALSE null",
        "OR or null",
        "ELSE else null",
        "NIL nil null",
        "IF if null",
        "PRINT print null",
        "IDENTIFIER WHILE null",
        "FUN fun null",
        "CLASS class null",
        "IDENTIFIER RETURN null",
        "IDENTIFIER IF null",
        "RETURN return null",
        "IDENTIFIER OR null",
        "SUPER super null",
        "IDENTIFIER TRUE null",
        "IDENTIFIER ELSE null",
        "FOR for null",
        "IDENTIFIER CLASS null",
        "IDENTIFIER PRINT null",
        "VAR var null",
        "IDENTIFIER VAR null",
        "EOF  null",
    ])]
    #[case("var greeting = \"Hello\"\nif (greeting == \"Hello\") {\n    return true\n} else {\n    return false\n}", vec![
        "VAR var null",
        "IDENTIFIER greeting null",
        "EQUAL = null",
        "STRING \"Hello\" Hello",
        "IF if null",
        "LEFT_PAREN ( null",
        "IDENTIFIER greeting null",
        "EQUAL_EQUAL == null",
        "STRING \"Hello\" Hello",
        "RIGHT_PAREN ) null",
        "LEFT_BRACE { null",
        "RETURN return null",
        "TRUE true null",
        "RIGHT_BRACE } null",
        "ELSE else null",
        "LEFT_BRACE { null",
        "RETURN return null",
        "FALSE false null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("var result = (a + b) > 7 or \"Success\" != \"Failure\" or x >= 5\nwhile (result) {\n    var counter = 0\n    counter = counter + 1\n    if (counter == 10) {\n        return nil\n    }\n}", vec![
        "VAR var null",
        "IDENTIFIER result null",
        "EQUAL = null",
        "LEFT_PAREN ( null",
        "IDENTIFIER a null",
        "PLUS + null",
        "IDENTIFIER b null",
        "RIGHT_PAREN ) null",
        "GREATER > null",
        "NUMBER 7 7.0",
        "OR or null",
        "STRING \"Success\" Success",
        "BANG_EQUAL != null",
        "STRING \"Failure\" Failure",
        "OR or null",
        "IDENTIFIER x null",
        "GREATER_EQUAL >= null",
        "NUMBER 5 5.0",
        "WHILE while null",
        "LEFT_PAREN ( null",
        "IDENTIFIER result null",
        "RIGHT_PAREN ) null",
        "LEFT_BRACE { null",
        "VAR var null",
        "IDENTIFIER counter null",
        "EQUAL = null",
        "NUMBER 0 0.0",
        "IDENTIFIER counter null",
        "EQUAL = null",
        "IDENTIFIER counter null",
        "PLUS + null",
        "NUMBER 1 1.0",
        "IF if null",
        "LEFT_PAREN ( null",
        "IDENTIFIER counter null",
        "EQUAL_EQUAL == null",
        "NUMBER 10 10.0",
        "RIGHT_PAREN ) null",
        "LEFT_BRACE { null",
        "RETURN return null",
        "NIL nil null",
        "RIGHT_BRACE } null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("baz bar", vec![
        "IDENTIFIER baz null",
        "IDENTIFIER bar null",
        "EOF  null",
    ])]
    #[case("_123baz f00 6ar foo bar", vec![
        "IDENTIFIER _123baz null",
        "IDENTIFIER f00 null",
        "NUMBER 6 6.0",
        "IDENTIFIER ar null",
        "IDENTIFIER foo null",
        "IDENTIFIER bar null",
        "EOF  null",
    ])]
    #[case("message = \"Hello, World!\"\nnumber = 123", vec![
        "IDENTIFIER message null",
        "EQUAL = null",
        "STRING \"Hello, World!\" Hello, World!",
        "IDENTIFIER number null",
        "EQUAL = null",
        "NUMBER 123 123.0",
        "EOF  null",
    ])]
    #[case("{\n// This is a complex test case\nstr1 = \"Test\"\nstr2 = \"Case\"\nnum1 = 100\nnum2 = 200.00\nresult = (str1 == str2) != ((num1 + num2) >= 300)\n}", vec![
        "LEFT_BRACE { null",
        "IDENTIFIER str1 null",
        "EQUAL = null",
        "STRING \"Test\" Test",
        "IDENTIFIER str2 null",
        "EQUAL = null",
        "STRING \"Case\" Case",
        "IDENTIFIER num1 null",
        "EQUAL = null",
        "NUMBER 100 100.0",
        "IDENTIFIER num2 null",
        "EQUAL = null",
        "NUMBER 200.00 200.0",
        "IDENTIFIER result null",
        "EQUAL = null",
        "LEFT_PAREN ( null",
        "IDENTIFIER str1 null",
        "EQUAL_EQUAL == null",
        "IDENTIFIER str2 null",
        "RIGHT_PAREN ) null",
        "BANG_EQUAL != null",
        "LEFT_PAREN ( null",
        "LEFT_PAREN ( null",
        "IDENTIFIER num1 null",
        "PLUS + null",
        "IDENTIFIER num2 null",
        "RIGHT_PAREN ) null",
        "GREATER_EQUAL >= null",
        "NUMBER 300 300.0",
        "RIGHT_PAREN ) null",
        "RIGHT_BRACE } null",
        "EOF  null",
    ])]
    #[case("44", vec![
        "NUMBER 44 44.0",
        "EOF  null",
    ])]
    #[case("2438.6541", vec![
        "NUMBER 2438.6541 2438.6541",
        "EOF  null",
    ])]
    #[case("19.0000", vec![
        "NUMBER 19.0000 19.0",
        "EOF  null",
    ])]
    #[case("(42+77) > 98 != (\"Success\" != \"Failure\") != (46 >= 83)", vec![
        "LEFT_PAREN ( null",
        "NUMBER 42 42.0",
        "PLUS + null",
        "NUMBER 77 77.0",
        "RIGHT_PAREN ) null",
        "GREATER > null",
        "NUMBER 98 98.0",
        "BANG_EQUAL != null",
        "LEFT_PAREN ( null",
        "STRING \"Success\" Success",
        "BANG_EQUAL != null",
        "STRING \"Failure\" Failure",
        "RIGHT_PAREN ) null",
        "BANG_EQUAL != null",
        "LEFT_PAREN ( null",
        "NUMBER 46 46.0",
        "GREATER_EQUAL >= null",
        "NUMBER 83 83.0",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("\"hello\"", vec![
        "STRING \"hello\" hello",
        "EOF  null",
    ])]
    #[case("\"hello\" , \"unterminated", vec![
        "STRING \"hello\" hello",
        "COMMA , null",
        "[line 1] Error: Unterminated string.",
        "EOF  null",
    ])]
    #[case("\"foo \tbar 123 // hello world!\"", vec![
        "STRING \"foo \tbar 123 // hello world!\" foo \tbar 123 // hello world!",
        "EOF  null",
    ])]
    #[case("(\"baz\"+\"bar\") != \"other_string\"", vec![
        "LEFT_PAREN ( null",
        "STRING \"baz\" baz",
        "PLUS + null",
        "STRING \"bar\" bar",
        "RIGHT_PAREN ) null",
        "BANG_EQUAL != null",
        "STRING \"other_string\" other_string",
        "EOF  null",
    ])]
    #[case(" ", vec![
        "EOF  null",
    ])]
    #[case(" \t\n ", vec![
        "EOF  null",
    ])]
    #[case("{\n\t}\n((\t+-\n*))", vec![
        "LEFT_BRACE { null",
        "RIGHT_BRACE } null",
        "LEFT_PAREN ( null",
        "LEFT_PAREN ( null",
        "PLUS + null",
        "MINUS - null",
        "STAR * null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("{ \t \n\t}\n((\t\n-<<=))", vec![
        "LEFT_BRACE { null",
        "RIGHT_BRACE } null",
        "LEFT_PAREN ( null",
        "LEFT_PAREN ( null",
        "MINUS - null",
        "LESS < null",
        "LESS_EQUAL <= null",
        "RIGHT_PAREN ) null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
    #[case("//Comment", vec![
        "EOF  null",
    ])]
    #[case("(///Unicode:£§᯽☺♣)", vec![
        "LEFT_PAREN ( null",
        "EOF  null",
    ])]
    #[case("/", vec![
        "SLASH / null",
        "EOF  null",
    ])]
    #[case("({(<=-+)})//Comment", vec![
        "LEFT_PAREN ( null",
        "LEFT_BRACE { null",
        "LEFT_PAREN ( null",
        "LESS_EQUAL <= null",
        "MINUS - null",
        "PLUS + null",
        "RIGHT_PAREN ) null",
        "RIGHT_BRACE } null",
        "RIGHT_PAREN ) null",
        "EOF  null",
    ])]
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

        let mut output = Vec::new();
        for sr in scanner.scan_tokens() {
            let s = match sr {
                ScanResult::Ignore => continue,
                ScanResult::Result(Ok(token)) => token.to_string(),
                ScanResult::Result(Err(e)) => e.to_string(),
            };
            output.push(s);
        }

        assert_eq!(output, expected_output);
    }
}
