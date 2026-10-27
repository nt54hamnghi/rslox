use std::iter::Peekable;

use color_eyre::eyre::OptionExt;

use crate::chunk::{Chunk, OpCode};
use crate::errors::Error;
use crate::scanner::token::{Token, TokenType};
use crate::value::Value;

pub fn compile(source: &str, chunk: &mut Chunk) -> Result<(), Error> {
    Parser::parse(source, chunk).map_err(Error::Compile)?;
    Ok(())
}

#[derive(Debug)]
struct Parser<'source, 'chunk> {
    source: &'source str,
    scanner: Peekable<Scanner<'source>>,
    chunk: &'chunk mut Chunk,
    previous: Token,
}

impl<'source, 'chunk> Parser<'source, 'chunk> {
    fn parse(source: &'source str, chunk: &'chunk mut Chunk) -> color_eyre::Result<()> {
        let mut scanner = Scanner::new(source).peekable();
        let mut parser = Self {
            source,
            scanner,
            chunk,
            previous: Token::not_a_token(),
        };
        parser.expression(0)
    }

    fn expression(&mut self, min_bp: u8) -> color_eyre::Result<()> {
        let token = self
            .scanner
            .next()
            .ok_or_eyre("unexpected end of token streams")?;
        self.previous = token;

        // parsing lhs
        match token.typ() {
            TokenType::NUMBER => self.number(),
            TokenType::EOF => return Ok(()),
            _ => todo!(),
        };

        loop {
            let op = *self
                .scanner
                .peek()
                .ok_or_eyre("unexpected end of token streams")?;

            let prec = match op.typ() {
                TokenType::EOF => break,
                _ => Precedence::from(op),
            };

            if let Some((lbp, rbp)) = prec.infix_binding_power() {
                if lbp < min_bp {
                    break;
                }

                // advance pass the operator
                self.scanner.next().unwrap();
                // parse rhs
                self.expression(rbp)?;

                let opcode = match op.typ() {
                    TokenType::PLUS => OpCode::OP_ADD,
                    TokenType::MINUS => OpCode::OP_SUBTRACT,
                    TokenType::SLASH => OpCode::OP_DIVIDE,
                    TokenType::STAR => OpCode::OP_MULTIPLY,
                    _ => unreachable!(),
                };

                self.emit_opcode(opcode);
            }
        }

        Ok(())
    }

    fn number(&mut self) {
        let token = self.previous;
        // the scanner only produces NUMBER tokens for valid numeric literals
        debug_assert_eq!(token.typ(), TokenType::NUMBER);
        let lexeme = token.lexeme(self.source);
        let value = lexeme.parse::<f64>().unwrap();
        self.emit_constant(value);
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line();
        debug_assert_ne!(line, 0);
        self.chunk.write_byte(byte, line);
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode as u8);
    }

    fn emit_bytes(&mut self, bytes: impl IntoIterator<Item = u8>) {
        for b in bytes {
            self.emit_byte(b);
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let line = self.previous.line();
        debug_assert_ne!(line, 0);
        self.chunk.write_constant(value, line);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Precedence {
    NONE = 1,
    ASSIGNMENT = 3,
    OR = 5,
    AND = 7,
    EQUALITY = 9,
    COMPARISON = 11,
    TERM = 13,
    FACTOR = 15,
    UNARY = 17,
    CALL = 19,
    PRIMARY = 21,
}

impl Precedence {
    fn prefix_binding_power(&self) -> ((), u8) {
        match self {
            Precedence::UNARY => {
                let bp = *self as u8;
                ((), bp)
            }
            _ => panic!("unknown prefix operator"),
        }
    }

    fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self {
            Precedence::ASSIGNMENT | Precedence::TERM | Precedence::FACTOR => {
                let bp = *self as u8;
                Some((bp, bp + 1))
            }
            _ => None,
        }
    }
}

impl From<Token> for Precedence {
    fn from(value: Token) -> Self {
        match value.typ() {
            TokenType::PLUS | TokenType::MINUS => Self::TERM,
            TokenType::STAR | TokenType::SLASH => Self::FACTOR,
            _ => Self::NONE,
        }
    }
}

/// Temporary wrapper that prints and skips scanner errors so we can focus on
/// parsing and compiling. Proper error handling will be implemented later.
#[derive(Debug)]
struct Scanner<'source> {
    scanner: crate::scanner::Scanner<'source>,
}

impl<'source> Scanner<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            scanner: crate::scanner::Scanner::new(source),
        }
    }
}

impl Iterator for Scanner<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        for result in self.scanner.by_ref() {
            match result {
                Ok(token) => return Some(token),
                Err(err) => eprintln!("{err}"),
            }
        }
        None
    }
}

impl std::iter::FusedIterator for Scanner<'_> {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Parser, Precedence};
    use crate::chunk::{Chunk, OpCode};

    #[rstest]
    #[case::none(Precedence::NONE, None)]
    #[case::assignment(Precedence::ASSIGNMENT, Some((3, 4)))]
    #[case::or(Precedence::OR, None)]
    #[case::and(Precedence::AND, None)]
    #[case::equality(Precedence::EQUALITY, None)]
    #[case::comparison(Precedence::COMPARISON, None)]
    #[case::term(Precedence::TERM, Some((13, 14)))]
    #[case::factor(Precedence::FACTOR, Some((15, 16)))]
    #[case::unary(Precedence::UNARY, None)]
    #[case::call(Precedence::CALL, None)]
    #[case::primary(Precedence::PRIMARY, None)]
    fn infix_binding_power_returns_expected_pair(
        #[case] prec: Precedence,
        #[case] expected: Option<(u8, u8)>,
    ) {
        assert_eq!(prec.infix_binding_power(), expected);
    }

    #[test]
    fn parse_empty_source_succeeds() {
        let mut chunk = Chunk::new();
        let result = Parser::parse("", &mut chunk);

        assert!(result.is_ok());
        assert!(chunk.code().is_empty());
        assert!(chunk.values().is_empty());
    }

    #[test]
    fn parse_single_number_emits_constant() {
        let mut chunk = Chunk::new();
        let result = Parser::parse("42", &mut chunk);

        assert!(result.is_ok());
        assert_eq!(chunk.code(), &[OpCode::OP_CONSTANT as u8, 0]);
        assert_eq!(chunk.values(), &[42.0]);
    }

    #[rstest]
    #[case::add("8 + 2", OpCode::OP_ADD)]
    #[case::sub("8 - 2", OpCode::OP_SUBTRACT)]
    #[case::mul("8 * 2", OpCode::OP_MULTIPLY)]
    #[case::div("8 / 2", OpCode::OP_DIVIDE)]
    fn parse_binary_operation_emits_constants_and_operator(
        #[case] source: &str,
        #[case] opcode: OpCode,
    ) {
        let mut chunk = Chunk::new();
        let result = Parser::parse(source, &mut chunk);

        assert!(result.is_ok());
        assert_eq!(
            chunk.code(),
            &[
                OpCode::OP_CONSTANT as u8,
                0,
                OpCode::OP_CONSTANT as u8,
                1,
                opcode as u8,
            ]
        );
        assert_eq!(chunk.values(), &[8.0, 2.0]);
    }

    #[rstest]
    #[case::mul_before_add("1 + 2 * 3", OpCode::OP_MULTIPLY, OpCode::OP_ADD)]
    #[case::div_before_add("1 + 2 / 3", OpCode::OP_DIVIDE, OpCode::OP_ADD)]
    #[case::mul_before_sub("1 - 2 * 3", OpCode::OP_MULTIPLY, OpCode::OP_SUBTRACT)]
    #[case::div_before_sub("1 - 2 / 3", OpCode::OP_DIVIDE, OpCode::OP_SUBTRACT)]
    fn parse_operator_precedence(
        #[case] source: &str,
        #[case] higher_prec: OpCode,
        #[case] lower_prec: OpCode,
    ) {
        let mut chunk = Chunk::new();
        let result = Parser::parse(source, &mut chunk);

        assert!(result.is_ok());
        assert_eq!(
            chunk.code(),
            &[
                OpCode::OP_CONSTANT as u8,
                0,
                OpCode::OP_CONSTANT as u8,
                1,
                OpCode::OP_CONSTANT as u8,
                2,
                higher_prec as u8,
                lower_prec as u8,
            ]
        );
        assert_eq!(chunk.values(), &[1.0, 2.0, 3.0]);
    }

    #[rstest]
    #[case::add("8 + 4 + 2", OpCode::OP_ADD, OpCode::OP_ADD)]
    #[case::sub("8 - 4 - 2", OpCode::OP_SUBTRACT, OpCode::OP_SUBTRACT)]
    #[case::mul("8 * 4 * 2", OpCode::OP_MULTIPLY, OpCode::OP_MULTIPLY)]
    #[case::div("8 / 4 / 2", OpCode::OP_DIVIDE, OpCode::OP_DIVIDE)]
    #[case::add_then_sub("8 + 4 - 2", OpCode::OP_ADD, OpCode::OP_SUBTRACT)]
    #[case::sub_then_add("8 - 4 + 2", OpCode::OP_SUBTRACT, OpCode::OP_ADD)]
    #[case::mul_then_div("8 * 4 / 2", OpCode::OP_MULTIPLY, OpCode::OP_DIVIDE)]
    #[case::div_then_mul("8 / 4 * 2", OpCode::OP_DIVIDE, OpCode::OP_MULTIPLY)]
    fn parse_operator_associativity(
        #[case] source: &str,
        #[case] first_op: OpCode,
        #[case] second_op: OpCode,
    ) {
        let mut chunk = Chunk::new();
        let result = Parser::parse(source, &mut chunk);

        assert!(result.is_ok());
        assert_eq!(
            chunk.code(),
            &[
                OpCode::OP_CONSTANT as u8,
                0,
                OpCode::OP_CONSTANT as u8,
                1,
                first_op as u8,
                OpCode::OP_CONSTANT as u8,
                2,
                second_op as u8,
            ]
        );
        assert_eq!(chunk.values(), &[8.0, 4.0, 2.0]);
    }
}
