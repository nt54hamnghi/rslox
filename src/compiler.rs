use std::iter::Peekable;

use crate::chunk::{Chunk, OpCode};
use crate::errors::LocationContext;
use crate::token::{Token, TokenType};
use crate::value::Value;

pub fn compile(source: &str, chunk: &mut Chunk) -> color_eyre::Result<()> {
    Parser::parse(source, chunk)
}

#[derive(Debug)]
struct Parser<'source, 'chunk> {
    source: &'source str,
    scanner: Peekable<Scanner<'source>>,
    chunk: &'chunk mut Chunk,
    current: Token,
}

impl<'source, 'chunk> Parser<'source, 'chunk> {
    fn parse(source: &'source str, chunk: &'chunk mut Chunk) -> color_eyre::Result<()> {
        let mut scanner = Scanner::new(source).peekable();
        let Some(current) = scanner.next() else {
            panic!("scanner terminated without producing an EOF token")
        };
        let mut parser = Self {
            source,
            scanner,
            chunk,
            current,
        };
        parser.expr(0)?;
        parser.emit_opcode(OpCode::OP_RETURN);
        Ok(())
    }

    /// Advances to the next token, then parses an expression with minimum binding power `min_bp`.
    /// Returns an error if the scanner is exhausted or parsing fails.
    fn advance_expr(&mut self, min_bp: u8) -> color_eyre::Result<()> {
        self.current = self
            .scanner
            .next()
            .expect("unexpected end of token streams");

        self.expr(min_bp)
    }

    fn expr(&mut self, min_bp: u8) -> color_eyre::Result<()> {
        let token = self.current;

        // parsing lhs
        match token.typ() {
            TokenType::EOF => return Ok(()),
            TokenType::NUMBER => self.number(),
            TokenType::STRING => self.string(),
            TokenType::NIL => self.emit_opcode(OpCode::OP_NIL),
            TokenType::TRUE => self.emit_opcode(OpCode::OP_TRUE),
            TokenType::FALSE => self.emit_opcode(OpCode::OP_FALSE),
            TokenType::LEFT_PAREN => self.grouping()?,
            _ => {
                if let Some(rule) = token.prefix_rule() {
                    let rbp = rule.prec as u8;
                    self.advance_expr(rbp)?;
                    self.emit_opcode(rule.opcode.0);
                }
            }
        };

        loop {
            let Some(token) = self.scanner.peek() else {
                break;
            };

            if let Some(Rule { opcode, prec }) = token.infix_rule() {
                let (lbp, rbp) = match prec {
                    Precedence::EQUALITY
                    | Precedence::COMPARISON
                    | Precedence::TERM
                    | Precedence::FACTOR => (prec as u8, prec as u8 + 1),
                    _ => unreachable!(),
                };

                if lbp < min_bp {
                    break;
                }

                // consumed the operator token
                self.scanner.next().unwrap();
                self.advance_expr(rbp)?;

                match opcode {
                    (first, None) => self.emit_opcode(first),
                    (first, Some(second)) => {
                        self.emit_opcode(first);
                        self.emit_opcode(second);
                    }
                };

                continue;
            }

            break;
        }

        Ok(())
    }

    fn grouping(&mut self) -> color_eyre::Result<()> {
        self.advance_expr(0)?;
        self.consume(TokenType::RIGHT_PAREN, "expect ')' after expression")?;
        Ok(())
    }

    fn number(&mut self) {
        let token = self.current;
        debug_assert_eq!(token.typ(), TokenType::NUMBER);

        let lexeme = token.lexeme(self.source);
        // the scanner produces NUMBER tokens only for valid numeric literals
        let number = lexeme.parse::<f64>().unwrap();
        self.emit_constant(number.into());
    }

    fn string(&mut self) {
        debug_assert_eq!(self.current.typ(), TokenType::STRING);

        let lexeme = self.current.lexeme(self.source);
        // trim the surrounding double quotes
        let content = lexeme[1..lexeme.len() - 1].to_owned();
        let string: &'static str = Box::leak(Box::new(content));
        self.emit_constant(string.into());
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> color_eyre::Result<()> {
        if self.scanner.next_if(|t| t.typ() == token_type).is_none() {
            return Err(self.error_at(self.current, msg));
        };
        Ok(())
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.current.line();
        debug_assert_ne!(line, 0);
        self.chunk.write_byte(byte, line);
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_byte(opcode as u8);
    }

    fn emit_constant(&mut self, value: Value) {
        let line = self.current.line();
        debug_assert_ne!(line, 0);
        self.chunk.write_constant(value, line);
    }

    fn error_at(&self, token: Token, msg: &str) -> color_eyre::Report {
        let span = match token.typ() {
            TokenType::EOF => "end",
            _ => token.lexeme(self.source),
        };

        msg.at_site(token.line(), span).into()
    }
}

impl Token {
    fn prefix_rule(&self) -> Option<Rule> {
        match self.typ() {
            TokenType::MINUS => Some(Rule {
                opcode: (OpCode::OP_NEGATE, None),
                prec: Precedence::UNARY,
            }),
            TokenType::BANG => Some(Rule {
                opcode: (OpCode::OP_NOT, None),
                prec: Precedence::UNARY,
            }),
            _ => None,
        }
    }

    fn infix_rule(&self) -> Option<Rule> {
        match self.typ() {
            TokenType::EQUAL_EQUAL => Some(Rule {
                opcode: (OpCode::OP_EQUAL, None),
                prec: Precedence::EQUALITY,
            }),
            TokenType::BANG_EQUAL => Some(Rule {
                opcode: (OpCode::OP_EQUAL, Some(OpCode::OP_NOT)),
                prec: Precedence::EQUALITY,
            }),
            TokenType::GREATER => Some(Rule {
                opcode: (OpCode::OP_GREATER, None),
                prec: Precedence::COMPARISON,
            }),
            TokenType::GREATER_EQUAL => Some(Rule {
                opcode: (OpCode::OP_LESS, Some(OpCode::OP_NOT)),
                prec: Precedence::COMPARISON,
            }),
            TokenType::LESS => Some(Rule {
                opcode: (OpCode::OP_LESS, None),
                prec: Precedence::COMPARISON,
            }),
            TokenType::LESS_EQUAL => Some(Rule {
                opcode: (OpCode::OP_GREATER, Some(OpCode::OP_NOT)),
                prec: Precedence::COMPARISON,
            }),
            TokenType::PLUS => Some(Rule {
                opcode: (OpCode::OP_ADD, None),
                prec: Precedence::TERM,
            }),
            TokenType::MINUS => Some(Rule {
                opcode: (OpCode::OP_SUBTRACT, None),
                prec: Precedence::TERM,
            }),
            TokenType::SLASH => Some(Rule {
                opcode: (OpCode::OP_DIVIDE, None),
                prec: Precedence::FACTOR,
            }),
            TokenType::STAR => Some(Rule {
                opcode: (OpCode::OP_MULTIPLY, None),
                prec: Precedence::FACTOR,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rule {
    opcode: (OpCode, Option<OpCode>),
    prec: Precedence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(unused)]
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

    use super::*;
    use crate::chunk::{Chunk, OpCode};
    use crate::token::{Token, TokenType};
    use crate::value::Value;

    #[test]
    fn parse_empty_source_succeeds() {
        let mut chunk = Chunk::new();
        let result = Parser::parse("", &mut chunk);

        assert!(result.is_ok());
        assert_eq!(chunk.code(), &[OpCode::OP_RETURN as u8]);
        assert!(chunk.values().is_empty());
    }

    #[test]
    fn parse_single_number_emits_constant() {
        let mut chunk = Chunk::new();
        let result = Parser::parse("42", &mut chunk);

        assert!(result.is_ok());
        assert_eq!(
            chunk.code(),
            &[OpCode::OP_CONSTANT as u8, 0, OpCode::OP_RETURN as u8]
        );
        assert_eq!(chunk.values(), &[Value::from(42.0)]);
    }

    #[test]
    fn parse_unary_minus_emits_negate() {
        let mut chunk = Chunk::new();
        let result = Parser::parse("-42", &mut chunk);

        assert!(result.is_ok());
        assert_eq!(
            chunk.code(),
            &[
                OpCode::OP_CONSTANT as u8,
                0,
                OpCode::OP_NEGATE as u8,
                OpCode::OP_RETURN as u8,
            ]
        );
        assert_eq!(chunk.values(), &[Value::from(42.0)]);
    }

    #[rstest]
    #[case::nil("!nil", &[OpCode::OP_NIL, OpCode::OP_NOT])]
    #[case::true_boolean("!true", &[OpCode::OP_TRUE, OpCode::OP_NOT])]
    #[case::false_boolean("!false", &[OpCode::OP_FALSE, OpCode::OP_NOT])]
    #[case::grouped("!(true)", &[OpCode::OP_TRUE, OpCode::OP_NOT])]
    #[case::nested("!!true", &[OpCode::OP_TRUE, OpCode::OP_NOT, OpCode::OP_NOT])]
    #[case::mixed("!true == false", &[OpCode::OP_TRUE, OpCode::OP_NOT, OpCode::OP_FALSE, OpCode::OP_EQUAL])]
    #[case::grouped_mixed("!(true == false)", &[OpCode::OP_TRUE, OpCode::OP_FALSE, OpCode::OP_EQUAL, OpCode::OP_NOT])]
    fn parse_logical_not_emits_opcodes(#[case] source: &str, #[case] opcodes: &[OpCode]) {
        let mut chunk = Chunk::new();
        Parser::parse(source, &mut chunk).unwrap();

        let mut expected: Vec<u8> = opcodes.iter().map(|opcode| *opcode as u8).collect();
        expected.push(OpCode::OP_RETURN as u8);

        assert_eq!(chunk.code(), expected);
        assert!(chunk.values().is_empty());
    }

    #[rstest]
    #[case::before_add("-8 + 2", OpCode::OP_ADD)]
    #[case::before_sub("-8 - 2", OpCode::OP_SUBTRACT)]
    #[case::before_mul("-8 * 2", OpCode::OP_MULTIPLY)]
    #[case::before_div("-8 / 2", OpCode::OP_DIVIDE)]
    fn parse_unary_operator_precedence(#[case] source: &str, #[case] binary_op: OpCode) {
        let mut chunk = Chunk::new();
        let result = Parser::parse(source, &mut chunk);

        assert!(result.is_ok());
        assert_eq!(
            chunk.code(),
            &[
                OpCode::OP_CONSTANT as u8,
                0,
                OpCode::OP_NEGATE as u8,
                OpCode::OP_CONSTANT as u8,
                1,
                binary_op as u8,
                OpCode::OP_RETURN as u8,
            ]
        );
        assert_eq!(chunk.values(), &[8.0, 2.0].map(Value::from));
    }

    #[rstest]
    #[case::add("8 + 2", &[OpCode::OP_ADD])]
    #[case::sub("8 - 2", &[OpCode::OP_SUBTRACT])]
    #[case::mul("8 * 2", &[OpCode::OP_MULTIPLY])]
    #[case::div("8 / 2", &[OpCode::OP_DIVIDE])]
    #[case::equal("8 == 2", &[OpCode::OP_EQUAL])]
    #[case::not_equal("8 != 2", &[OpCode::OP_EQUAL, OpCode::OP_NOT])]
    #[case::greater("8 > 2", &[OpCode::OP_GREATER])]
    #[case::greater_equal("8 >= 2", &[OpCode::OP_LESS, OpCode::OP_NOT])]
    #[case::less("8 < 2", &[OpCode::OP_LESS])]
    #[case::less_equal("8 <= 2", &[OpCode::OP_GREATER, OpCode::OP_NOT])]
    fn parse_binary_operation_emits_constants_and_operator(
        #[case] source: &str,
        #[case] opcodes: &[OpCode],
    ) {
        let mut chunk = Chunk::new();
        let result = Parser::parse(source, &mut chunk);

        assert!(result.is_ok());
        let mut expected = vec![OpCode::OP_CONSTANT as u8, 0, OpCode::OP_CONSTANT as u8, 1];
        expected.extend(opcodes.iter().map(|opcode| *opcode as u8));
        expected.push(OpCode::OP_RETURN as u8);
        assert_eq!(chunk.code(), expected);
        assert_eq!(chunk.values(), &[8.0, 2.0].map(Value::from));
    }

    #[rstest]
    #[case::mul_before_add("1 + 2 * 3", OpCode::OP_MULTIPLY, OpCode::OP_ADD)]
    #[case::div_before_add("1 + 2 / 3", OpCode::OP_DIVIDE, OpCode::OP_ADD)]
    #[case::mul_before_sub("1 - 2 * 3", OpCode::OP_MULTIPLY, OpCode::OP_SUBTRACT)]
    #[case::div_before_sub("1 - 2 / 3", OpCode::OP_DIVIDE, OpCode::OP_SUBTRACT)]
    #[case::add_before_less("1 < 2 + 3", OpCode::OP_ADD, OpCode::OP_LESS)]
    #[case::sub_before_greater("1 > 2 - 3", OpCode::OP_SUBTRACT, OpCode::OP_GREATER)]
    #[case::less_before_equal("1 == 2 < 3", OpCode::OP_LESS, OpCode::OP_EQUAL)]
    #[case::greater_before_equal("1 == 2 > 3", OpCode::OP_GREATER, OpCode::OP_EQUAL)]
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
                OpCode::OP_RETURN as u8,
            ]
        );
        assert_eq!(chunk.values(), &[1.0, 2.0, 3.0].map(Value::from));
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
    #[case::equal("8 == 4 == 2", OpCode::OP_EQUAL, OpCode::OP_EQUAL)]
    #[case::less("8 < 4 < 2", OpCode::OP_LESS, OpCode::OP_LESS)]
    #[case::greater("8 > 4 > 2", OpCode::OP_GREATER, OpCode::OP_GREATER)]
    #[case::less_then_greater("8 < 4 > 2", OpCode::OP_LESS, OpCode::OP_GREATER)]
    #[case::comparison_before_equal("8 < 4 == 2", OpCode::OP_LESS, OpCode::OP_EQUAL)]
    #[case::add_before_comparison("8 + 4 < 2", OpCode::OP_ADD, OpCode::OP_LESS)]
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
                OpCode::OP_RETURN as u8,
            ]
        );
        assert_eq!(chunk.values(), &[8.0, 4.0, 2.0].map(Value::from));
    }

    #[rstest]
    #[case::single_number("(8)", &[OpCode::OP_CONSTANT as u8, 0, OpCode::OP_RETURN as u8], &[8.0])]
    #[case::nested_group("((8))", &[OpCode::OP_CONSTANT as u8, 0, OpCode::OP_RETURN as u8], &[8.0])]
    #[case::precedence(
                "(8 + 4) * 2",
                &[OpCode::OP_CONSTANT as u8, 0, OpCode::OP_CONSTANT as u8, 1,
                  OpCode::OP_ADD as u8, OpCode::OP_CONSTANT as u8, 2, OpCode::OP_MULTIPLY as u8,
                  OpCode::OP_RETURN as u8],
                &[8.0, 4.0, 2.0],
            )]
    #[case::associativity(
                "8 - (4 - 2)",
                &[OpCode::OP_CONSTANT as u8, 0, OpCode::OP_CONSTANT as u8, 1,
                  OpCode::OP_CONSTANT as u8, 2, OpCode::OP_SUBTRACT as u8, OpCode::OP_SUBTRACT as u8,
                  OpCode::OP_RETURN as u8],
                &[8.0, 4.0, 2.0],
            )]
    #[case::unary(
                "-(8 + 4)",
                &[OpCode::OP_CONSTANT as u8, 0, OpCode::OP_CONSTANT as u8, 1,
                  OpCode::OP_ADD as u8, OpCode::OP_NEGATE as u8, OpCode::OP_RETURN as u8],
                &[8.0, 4.0],
            )]
    fn parse_grouped_expression(
        #[case] source: &str,
        #[case] expected_code: &[u8],
        #[case] expected_values: &[f64],
    ) {
        let mut chunk = Chunk::new();
        let result = Parser::parse(source, &mut chunk);

        assert!(result.is_ok());
        assert_eq!(chunk.code(), expected_code);
        let expected_values: Vec<Value> =
            expected_values.iter().copied().map(Value::from).collect();
        assert_eq!(chunk.values(), expected_values);
    }

    #[rstest]
    #[case::minus(TokenType::MINUS, OpCode::OP_NEGATE, Precedence::UNARY)]
    #[case::bang(TokenType::BANG, OpCode::OP_NOT, Precedence::UNARY)]
    fn token_prefix_rule_returns_opcode_and_precedence(
        #[case] typ: TokenType,
        #[case] opcode: OpCode,
        #[case] precedence: Precedence,
    ) {
        let token = Token::new(0, 1, typ, 1);

        assert_eq!(
            token.prefix_rule(),
            Some(Rule {
                opcode: (opcode, None),
                prec: precedence,
            })
        );
    }

    #[rstest]
    #[case::equal(TokenType::EQUAL_EQUAL, (OpCode::OP_EQUAL, None), Precedence::EQUALITY)]
    #[case::not_equal(TokenType::BANG_EQUAL, (OpCode::OP_EQUAL, Some(OpCode::OP_NOT)), Precedence::EQUALITY)]
    #[case::greater(TokenType::GREATER, (OpCode::OP_GREATER, None), Precedence::COMPARISON)]
    #[case::greater_equal(TokenType::GREATER_EQUAL, (OpCode::OP_LESS, Some(OpCode::OP_NOT)), Precedence::COMPARISON)]
    #[case::less(TokenType::LESS, (OpCode::OP_LESS, None), Precedence::COMPARISON)]
    #[case::less_equal(TokenType::LESS_EQUAL, (OpCode::OP_GREATER, Some(OpCode::OP_NOT)), Precedence::COMPARISON)]
    #[case::plus(TokenType::PLUS, (OpCode::OP_ADD, None), Precedence::TERM)]
    #[case::minus(TokenType::MINUS, (OpCode::OP_SUBTRACT, None), Precedence::TERM)]
    #[case::slash(TokenType::SLASH, (OpCode::OP_DIVIDE, None), Precedence::FACTOR)]
    #[case::star(TokenType::STAR, (OpCode::OP_MULTIPLY, None), Precedence::FACTOR)]
    fn token_infix_rule_returns_opcode_and_precedence(
        #[case] typ: TokenType,
        #[case] opcode: (OpCode, Option<OpCode>),
        #[case] precedence: Precedence,
    ) {
        let token = Token::new(0, 1, typ, 1);

        assert_eq!(
            token.infix_rule(),
            Some(Rule {
                opcode,
                prec: precedence,
            })
        );
    }
}
