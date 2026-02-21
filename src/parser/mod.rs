use std::iter::Peekable;
use std::vec;

use crate::Value;
use crate::error::Report;
use crate::parser::expr::{AstNode, Binary, Grouping, Literal, Unary};
use crate::scanner::token::TokenType::{
    Bang, BangEqual, Eof, EqualEqual, False, Greater, GreaterEqual, LeftParen, Less, LessEqual,
    Minus, Nil, Number, Plus, RightParen, Slash, Star, String as Str, True,
};
use crate::scanner::token::{Token, TokenType};

pub mod expr;
pub mod printer;

pub struct Parser {
    tokens: Peekable<vec::IntoIter<Token>>,
}

impl From<Vec<Token>> for Parser {
    fn from(value: Vec<Token>) -> Self {
        Self {
            tokens: value.into_iter().peekable(),
        }
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<AstNode, Report> {
        self.expression()
    }

    /// expression → equality ;
    fn expression(&mut self) -> Result<AstNode, Report> {
        self.equality()
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<AstNode, Report> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.next_match(&[BangEqual, EqualEqual]) {
            let right = self.comparison()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<AstNode, Report> {
        let mut expr = self.term()?;

        while let Some(operator) = self.next_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let right = self.term()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<AstNode, Report> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.next_match(&[Minus, Plus]) {
            let right = self.factor()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<AstNode, Report> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.next_match(&[Slash, Star]) {
            let right = self.unary()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// unary → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<AstNode, Report> {
        if let Some(operator) = self.next_match(&[Bang, Minus]) {
            let right = self.unary()?;
            return Ok(Unary::new(operator, right).into());
        }

        self.primary()
    }

    /// primary → NUMBER | STRING | "true" | "false" | "nil"| "(" expression ")" ;
    fn primary(&mut self) -> Result<AstNode, Report> {
        if self.next_if(True).is_some() {
            let val = Value::from(true);
            return Ok(Literal::from(val).into());
        }

        if self.next_if(False).is_some() {
            let val = Value::from(false);
            return Ok(Literal::from(val).into());
        }

        if self.next_if(Nil).is_some() {
            let val = Value::Nil;
            return Ok(Literal::from(val).into());
        }

        if let Some(token) = self.next_match(&[Number, Str]) {
            let value = token.literal.expect("literal value for token");
            return Ok(Literal::from(value).into());
        }

        if self.next_if(LeftParen).is_some() {
            let expr = self.expression()?;
            self.next_ok(RightParen, "Expect ')' after expression".into())?;
            return Ok(Grouping::new(expr).into());
        }

        Err(self.error("Expect expression".into()))
    }

    /// Consumes the next token if it matches any of the given types.
    ///
    /// Returns
    /// - `Some(&Token)` if a match is found, consuming the token.
    /// - `None` if no match is found or if at end of tokens.
    fn next_match(&mut self, types: &[TokenType]) -> Option<Token> {
        for tt in types {
            let next = self.next_if(*tt);
            if next.is_some() {
                return next;
            }
        }

        None
    }

    /// Consumes the next token if it matches the given type.
    ///
    /// Returns:
    /// - `Some(Token)` if the next token matches, consuming it.
    /// - `None` if the next token doesn't match or if at end of tokens.
    fn next_if(&mut self, tt: TokenType) -> Option<Token> {
        if self.is_at_end() {
            return None;
        }
        self.tokens.next_if(|token| token.typ == tt)
    }

    /// Consumes the next token if it matches the given type, or returns an error.
    ///
    /// Returns:
    /// - `Ok(Token)` if the next token matches the given type, consuming it.
    /// - `Err(Report)` if the next token doesn't match or if at end of tokens.
    fn next_ok(&mut self, tt: TokenType, message: String) -> Result<Token, Report> {
        self.next_if(tt).ok_or(self.error(message))
    }

    /// Checks if the parser has reached the end of the token stream.
    ///
    /// Returns `true` if the current token is an EOF token or
    /// there are no more tokens in the token list.
    fn is_at_end(&mut self) -> bool {
        self.tokens.peek().is_none_or(|t| t.typ == Eof)
    }

    fn error(&mut self, message: String) -> Report {
        let token = self.tokens.peek().expect("expected a token");
        Report::error_at_token(token, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::printer::AstPrinter;
    use crate::scanner::{ScanItem, Scanner};
    use rstest::rstest;

    #[rstest]
    #[case(r#""bar"!="hello""#, "(!= bar hello)")]
    #[case(r#""foo" == "foo""#, "(== foo foo)")]
    #[case("60 == 83", "(== 60.0 83.0)")]
    #[case(
        "(85 != 50) == ((-58 + 98) >= (89 * 74))",
        "(== (group (!= 85.0 50.0)) (group (>= (group (+ (- 58.0) 98.0)) (group (* 89.0 74.0)))))"
    )]
    #[case("97 > 65", "(> 97.0 65.0)")]
    #[case("32 <= 129", "(<= 32.0 129.0)")]
    #[case("97 < 129 < 161", "(< (< 97.0 129.0) 161.0)")]
    #[case(
        "(83 - 44) >= -(30 / 52 + 28)",
        "(>= (group (- 83.0 44.0)) (- (group (+ (/ 30.0 52.0) 28.0))))"
    )]
    #[case(r#""hello" + "world""#, "(+ hello world)")]
    #[case("66 - 25 * 66 - 65", "(- (- 66.0 (* 25.0 66.0)) 65.0)")]
    #[case("18 + 92 - 12 / 34", "(- (+ 18.0 92.0) (/ 12.0 34.0))")]
    #[case(
        "(-90 + 67) * (14 * 41) / (37 + 93)",
        "(/ (* (group (+ (- 90.0) 67.0)) (group (* 14.0 41.0))) (group (+ 37.0 93.0)))"
    )]
    #[case("20 * 49 / 16", "(/ (* 20.0 49.0) 16.0)")]
    #[case("50 / 88 / 65", "(/ (/ 50.0 88.0) 65.0)")]
    #[case("35 * 33 * 58 / 19", "(/ (* (* 35.0 33.0) 58.0) 19.0)")]
    #[case(
        "(76 * -39 / (16 * 62))",
        "(group (/ (* 76.0 (- 39.0)) (group (* 16.0 62.0))))"
    )]
    #[case("!false", "(! false)")]
    #[case("-63", "(- 63.0)")]
    #[case("!!false", "(! (! false))")]
    #[case("(!!(false))", "(group (! (! (group false))))")]
    #[case(r#""baz quz""#, "baz quz")]
    #[case(r#""'world'""#, "'world'")]
    #[case(r#""// world""#, "// world")]
    #[case(r#""21""#, "21")]
    #[case("57", "57.0")]
    #[case("0.0", "0.0")]
    #[case("86.63", "86.63")]
    #[case("true", "true")]
    #[case("false", "false")]
    #[case("nil", "nil")]
    fn test_parser(#[case] input: &str, #[case] expected_output: &str) {
        let tokens = Scanner::new(&input)
            .scan_tokens()
            .filter_map(|r| match r {
                Ok(ScanItem::Token(tkn)) => Some(tkn),
                Ok(ScanItem::Ignore) => None,
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let mut parser = Parser::from(tokens);
        let expr = parser.parse().unwrap();
        let expr_str = AstPrinter.print(&expr);
        assert_eq!(expected_output, expr_str)
    }
}
