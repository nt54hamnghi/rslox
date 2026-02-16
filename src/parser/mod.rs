use std::iter::Peekable;
use std::vec;

use crate::parser::expr::{Ast, Literal};
use crate::scanner::token::TokenType::{
    Bang, BangEqual, Eof, EqualEqual, False, Greater, GreaterEqual, LeftParen, Less, LessEqual,
    Minus, Nil, Number, Plus, RightParen, Slash, Star, String, True,
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
    pub fn parse(&mut self) -> Ast {
        self.expression()
    }

    /// expression → equality ;
    fn expression(&mut self) -> Ast {
        self.equality()
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Ast {
        let mut expr = self.comparison();

        while let Some(operator) = self.next_match(&[BangEqual, EqualEqual]) {
            let right = self.comparison();
            expr = Ast::binary(expr, operator, right);
        }

        expr
    }

    /// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Ast {
        let mut expr = self.term();

        while let Some(operator) = self.next_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let right = self.term();
            expr = Ast::binary(expr, operator, right);
        }

        expr
    }

    /// term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Ast {
        let mut expr = self.factor();

        while let Some(operator) = self.next_match(&[Minus, Plus]) {
            let right = self.factor();
            expr = Ast::binary(expr, operator, right);
        }

        expr
    }

    /// factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Ast {
        let mut expr = self.unary();

        while let Some(operator) = self.next_match(&[Slash, Star]) {
            let right = self.unary();
            expr = Ast::binary(expr, operator, right);
        }

        expr
    }

    /// unary → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Ast {
        if let Some(operator) = self.next_match(&[Bang, Minus]) {
            let right = self.unary();
            return Ast::unary(operator, right);
        }

        self.primary()
    }

    /// primary → NUMBER | STRING | "true" | "false" | "nil"| "(" expression ")" ;
    fn primary(&mut self) -> Ast {
        if self.next_if(True).is_some() {
            return Ast::literal(Literal::from(true));
        }

        if self.next_if(False).is_some() {
            return Ast::literal(Literal::from(false));
        }

        if self.next_if(Nil).is_some() {
            return Ast::literal(Literal::Nil);
        }

        if let Some(token) = self.next_match(&[Number, String]) {
            let value = token.literal.expect("literal value for token");
            return Ast::literal(value.into());
        }

        if self.next_if(LeftParen).is_some() {
            let expr = self.expression();
            self.next_if(RightParen)
                .expect("expected ')' after expression");
            return Ast::grouping(expr);
        }

        panic!("expected expression")
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

    /// Checks if the parser has reached the end of the token stream.
    ///
    /// Returns `true` if the current token is an EOF token or
    /// there are no more tokens in the token list.
    fn is_at_end(&mut self) -> bool {
        self.tokens.peek().is_none_or(|t| t.typ == Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::printer::AstPrinter;
    use crate::scanner::{ScanResult, Scanner};
    use rstest::rstest;

    #[rstest]
    #[case("57", "57.0")]
    #[case("0.0", "0.0")]
    #[case("86.63", "86.63")]
    #[case(r#""baz quz""#, "baz quz")]
    #[case(r#""'world'""#, "'world'")]
    #[case(r#""// world""#, "// world")]
    #[case(r#""21""#, "21")]
    #[case("true", "true")]
    #[case("false", "false")]
    #[case("nil", "nil")]
    fn test_parser(#[case] input: &str, #[case] expected_output: &str) {
        let tokens = Scanner::new(&input)
            .scan_tokens()
            .filter_map(|s| match s {
                ScanResult::Result(token) => token.ok(),
                ScanResult::Ignore => None,
            })
            .collect::<Vec<_>>();

        let mut parser = Parser::from(tokens);
        let expr_str = AstPrinter.print(parser.parse());
        assert_eq!(expected_output, expr_str)
    }
}
