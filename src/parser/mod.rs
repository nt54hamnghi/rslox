use std::iter::Peekable;
use std::vec;

use crate::Value;
use crate::error::StaticError;
use crate::parser::expr::{Binary, ExprNode, Grouping, Literal, Unary, Variable};
use crate::parser::stmt::{Expression, Print, StmtNode, Var};
use crate::scanner::token::{Token, TokenType};

pub mod expr;
pub mod printer;
pub mod stmt;

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
    /// Parses the full token stream as a sequence of statements until EOF.
    ///
    /// Returns:
    /// - `Ok(Vec<StmtNode>)` with all parsed statements.
    /// - `Err(Report)` when any statement cannot be parsed.
    pub fn parse(&mut self) -> Result<Vec<StmtNode>, StaticError> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            let s = self.declaration()?;
            stmts.push(s);
        }

        Ok(stmts)
    }

    /// Parses a single expression from the current parser position.
    ///
    /// This is used for expression-only entry points (for example, parse/eval
    /// commands that do not expect statement wrappers).
    ///
    /// Returns:
    /// - `Ok(ExprNode)` for a successfully parsed expression.
    /// - `Err(Report)` if expression parsing fails.
    pub fn parse_expression(&mut self) -> Result<ExprNode, StaticError> {
        self.expression()
    }

    #[allow(unused)]
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.next_if(TokenType::Semicolon).is_some() {
                return;
            }

            if matches!(
                self.tokens
                    .peek()
                    .expect("loop guard ensures next token exists and is not EOF")
                    .typ,
                TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Return
                    | TokenType::Print
            ) {
                return;
            }

            self.tokens.next();
        }
    }

    fn declaration(&mut self) -> Result<StmtNode, StaticError> {
        if self.next_if(TokenType::Var).is_some() {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<StmtNode, StaticError> {
        let name = self.next_ok(TokenType::Identifier, "Expect variable name.".into())?;

        let mut init = None;
        if self.next_if(TokenType::Equal).is_some() {
            init = Some(self.expression()?);
        }

        self.expect_semicolon()?;

        Ok(Var::new(name, init).into())
    }

    fn statement(&mut self) -> Result<StmtNode, StaticError> {
        if self.next_if(TokenType::Print).is_some() {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<StmtNode, StaticError> {
        let expr = self.expression()?;
        self.expect_semicolon()?;

        Ok(Print::new(expr).into())
    }

    fn expression_statement(&mut self) -> Result<StmtNode, StaticError> {
        let expr = self.expression()?;
        self.expect_semicolon()?;

        Ok(Expression::new(expr).into())
    }

    /// expression → equality ;
    fn expression(&mut self) -> Result<ExprNode, StaticError> {
        self.equality()
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<ExprNode, StaticError> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.next_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let right = self.comparison()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<ExprNode, StaticError> {
        let mut expr = self.term()?;

        while let Some(operator) = self.next_match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<ExprNode, StaticError> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.next_match(&[TokenType::Minus, TokenType::Plus]) {
            let right = self.factor()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<ExprNode, StaticError> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.next_match(&[TokenType::Slash, TokenType::Star]) {
            let right = self.unary()?;
            expr = Binary::new(expr, operator, right).into();
        }

        Ok(expr)
    }

    /// unary → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<ExprNode, StaticError> {
        if let Some(operator) = self.next_match(&[TokenType::Bang, TokenType::Minus]) {
            let right = self.unary()?;
            return Ok(Unary::new(operator, right).into());
        }

        self.primary()
    }

    /// primary → NUMBER | STRING | "true" | "false" | "nil"| "(" expression ")" ;
    fn primary(&mut self) -> Result<ExprNode, StaticError> {
        if self.next_if(TokenType::True).is_some() {
            let val = Value::from(true);
            return Ok(Literal::from(val).into());
        }

        if self.next_if(TokenType::False).is_some() {
            let val = Value::from(false);
            return Ok(Literal::from(val).into());
        }

        if self.next_if(TokenType::Nil).is_some() {
            let val = Value::Nil;
            return Ok(Literal::from(val).into());
        }

        if let Some(token) = self.next_match(&[TokenType::Number, TokenType::String]) {
            let value = token.literal.expect("literal value for token");
            return Ok(Literal::from(value).into());
        }

        if self.next_if(TokenType::LeftParen).is_some() {
            let expr = self.expression()?;
            self.next_ok(TokenType::RightParen, "Expect ')' after expression".into())?;
            return Ok(Grouping::new(expr).into());
        }

        if let Some(name) = self.next_if(TokenType::Identifier) {
            return Ok(Variable::new(name).into());
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
    fn next_ok(&mut self, tt: TokenType, message: String) -> Result<Token, StaticError> {
        self.next_if(tt).ok_or(self.error(message))
    }

    /// Consumes a required trailing semicolon token.
    ///
    /// Returns:
    /// - `Ok(Token)` when the next token is `;`, consuming it.
    /// - `Err(Report)` when `;` is missing.
    fn expect_semicolon(&mut self) -> Result<Token, StaticError> {
        self.next_ok(TokenType::Semicolon, "Expect ';' after value.".into())
    }

    /// Checks if the parser has reached the end of the token stream.
    ///
    /// Returns `true` if the current token is an EOF token or
    /// there are no more tokens in the token list.
    fn is_at_end(&mut self) -> bool {
        self.tokens.peek().is_none_or(|t| t.typ == TokenType::Eof)
    }

    fn error(&mut self, message: String) -> StaticError {
        let token = self.tokens.peek().expect("expected a token");
        StaticError::error_at_token(token, message)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::parser::printer::AstPrinter;
    use crate::parser::stmt::StmtNode;
    use crate::scanner::{ScanItem, Scanner};

    fn scan(input: &str) -> Vec<Token> {
        Scanner::new(input)
            .scan_tokens()
            .filter_map(|r| match r {
                Ok(ScanItem::Token(tkn)) => Some(tkn),
                Ok(ScanItem::Ignore) => None,
                Err(_) => None,
            })
            .collect()
    }

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
        let tokens = scan(input);
        let mut parser = Parser::from(tokens);
        let expr = parser.expression().unwrap();

        let expr_str = AstPrinter.print(&expr);
        assert_eq!(expected_output, expr_str)
    }

    fn parse_program(input: &str) -> Result<Vec<StmtNode>, StaticError> {
        let tokens = scan(input);
        let mut parser = Parser::from(tokens);
        parser.parse()
    }

    fn render_stmt(stmt: &StmtNode) -> String {
        match stmt {
            StmtNode::Print(print) => format!("print {}", AstPrinter.print(&*print.expr)),
            StmtNode::Expression(expression) => AstPrinter.print(&*expression.expr),
            StmtNode::Var(_var) => todo!(),
        }
    }

    #[test]
    fn test_parse_multiple_statements_single_line_and_multiline() {
        let program = r#"
            print "baz"; print false;
            print true;
            print "bar"; print 76;
        "#;

        let statements = parse_program(program).expect("Expected a valid program");
        let actual = statements.iter().map(render_stmt).collect::<Vec<_>>();
        let expected = vec![
            "print baz",
            "print false",
            "print true",
            "print bar",
            "print 76.0",
        ];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_print_requires_expression() {
        let err = parse_program("print;").expect_err("expected parse error");
        assert_eq!("[line 1] Error at ';': Expect expression", err.to_string());
    }
}
