use super::token::{Token, TokenKind};
use super::lexer;
use super::errors::{Errors};
use super::ast::{Program, Statement, LetStatement, ReturnStatement, 
                 Expression, ExpressionStatement, Precedence,
                 Identifier, Integer, Bool,LParen, IfExpression,
                 FunctionLiteral, PrefixExpression, InfixExpression};

#[derive(Debug, Clone)]
pub struct Parser<'a>  {
    lexer: lexer::Lexer<'a>,
    current_token: Token,
    next_token: Token,
}

impl<'a>  Parser<'a>  {
    pub fn new(l: lexer::Lexer<'a>) -> Self {
        // Goだと初期化時に省略するが、rustではできないためはじめにやる。

        let mut p = Parser{
            lexer: l,
            current_token: Token{token_type: TokenKind::DEFAULT, literal: "default".to_string() },
            next_token: Token{token_type: TokenKind::DEFAULT, literal: "default".to_string() },
        };
        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.current_token = self.next_token.clone(); //借用権問題でcloneする。
        self.next_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Result<Program, Errors> {
        let mut statements: Vec<Statement> = vec![];

        // eofになるまでstatementsを配列に入れる。
        while !self.is_current_token(TokenKind::EOF){
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.next_token();
        };
        // 読んだ式をprogramに入れて返す。
        Ok(Program {statements: statements})
    }

    fn parse_statement(&mut self) -> Result<Statement, Errors> {
        match self.current_token.token_type {
            TokenKind::LET => {
                Ok(Statement::LetStatement(self.parse_let_statement()?))
            },
            TokenKind::RETURN => {
                Ok(Statement::ReturnStatement(self.parse_return_statement()?))
            },
            _ => {
                Ok(Statement::ExpressionStatement(self.parse_expression_statement()?))
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement, Errors> {
        let identifier = self.next_token.clone();

        if !self.expect_next_token(TokenKind::IDENT) {
            return Err(Errors::TokenInvalid(self.next_token.clone()))
        }

        if !self.expect_next_token(TokenKind::ASSIGN) {
            return Err(Errors::TokenInvalid(self.next_token.clone()))
        }

        // セミコロンまでの読み飛ばしをしてからstatementを定義して返す。
        while !self.is_current_token(TokenKind::SEMICOLON) {
            self.next_token()
        }
        let stmt = LetStatement {
            identifier: Identifier{
                value: identifier.literal
            }
        };
        return Ok(stmt)
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatement, Errors> {
        let identifier = self.next_token.clone();
        let stmt = ReturnStatement {
            identifier: Identifier{
                value: identifier.literal
            }
        };
        // セミコロンまでの読み飛ばしをしてからstatementを定義して返す。
        while !self.is_current_token(TokenKind::SEMICOLON) {
            self.next_token()
        }
        return Ok(stmt)
    }

    fn parse_expression_statement(&mut self) -> Result<ExpressionStatement, Errors> {
        let expression = self.parse_expression(Precedence::LOWEST)?;
        if self.is_next_token(TokenKind::SEMICOLON) {
            self.next_token()
        }
        return Ok(ExpressionStatement{expression: expression})
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, Errors> {
        let mut exp = match self.current_token.token_type {
            TokenKind::IDENT => Expression::Identifier(self.parse_identifier()?),
            TokenKind::INT => Expression::Integer(self.parse_integer()?),
            TokenKind::TRUE => Expression::Bool(Bool{value: true}),
            TokenKind::FALSE => Expression::Bool(Bool{value: false}),
            TokenKind::IF => Expression::IfExpression(self.parse_if_expression()?),
            TokenKind::FUNCTION => Expression::FunctionLiteral(self.parse_function_expression()?),
            TokenKind::BANG => Expression::PrefixExpression(self.parse_prefix_expression()?),
            TokenKind::MINUS => Expression::PrefixExpression(self.parse_prefix_expression()?),
            TokenKind::LPAREN => self.parse_grouped_expression()?,
            _ => return Err(Errors::TokenInvalid(self.current_token.clone()))
        };
        while !self.is_next_token(TokenKind::SEMICOLON) && precedence < self.next_precedence() {
            match self.next_token.token_type {
                TokenKind::PLUS => {
                    //since operator must be set in current position,
                    //so token must be read once forward.
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::MINUS => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::SLASH => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::ASTERISK => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::EQ => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::NotEq => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::LT => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::GT => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                _ => {
                    return Ok(exp);                
                }
            }
        }
        return Ok(exp)
    }

    fn parse_identifier(&mut self) -> Result<Identifier, Errors> {
        return Ok(Identifier{value: self.current_token.literal.to_string()})
    }

    fn parse_integer(&mut self) -> Result<Integer, Errors> {
        return Ok(Integer{value: self.current_token.literal.to_string()})
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression, Errors> {
        let current_token = self.current_token.literal.to_string();
        self.next_token();
        let lparen = self.parse_expression(Precedence::LOWEST)?;
        if self.expect_next_token(TokenKind::RPAREN) {
             return Ok(lparen)
        } else {
            panic!()
        }  
}

    fn parse_if_expression(&mut self) ->  Result<IfExpression, Errors> {
        if !self.is_next_token(TokenKind::LPAREN) {
            println!("TokenKind should be LPAREN but actually is {:?}",self.next_token.token_type)
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST);
        if !self.expect_next_token(TokenKind::RPAREN) {
            println!("TokenKind should be RPAREN but actually is {:?}",self.next_token.token_type)
        }
        if !self.expect_next_token(TokenKind::LBRACE) {
            println!("TokenKind should be LBRACE but actually is {:?}",self.next_token.token_type)
        }


        let expression = IfExpression{
                            condition: Box::new(condition?),
                            consequence: Box::new(self.parse_block_statements(TokenKind::LBRACE)?),
                            alternative: Box::new(self.alternative()?),                           
        };
        Ok(expression)
    }

    fn parse_block_statements(&mut self, token_kind: TokenKind) -> Result<Statement, Errors> {
        self.next_token();
        let mut statements: Vec<Statement> = vec![];
        while !self.is_current_token(TokenKind::RBRACE) && !self.is_current_token(TokenKind::EOF) {
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.next_token();
        }
        Ok(Statement::Block(statements))
    }

    fn alternative(&mut self) -> Result<Statement, Errors> {
        if self.is_next_token(TokenKind::ELSE) {
        self.next_token();
        if self.expect_next_token(TokenKind::LBRACE) {
            Ok(self.parse_block_statements(TokenKind::LBRACE))?
        }else {
            return Err(Errors::TokenInvalid(self.current_token.clone()))
        }
    } else {
        return Err(Errors::TokenInvalid(self.current_token.clone()))
        }
    }

    fn parse_function_expression(&mut self) -> Result<FunctionLiteral, Errors> {
        if self.expect_next_token(TokenKind::LPAREN) {
            println!("TokenKind should be LPAREN but actually is {:?}",self.next_token.token_type)            
        }
        let parameters = self.parse_function_parameters()?;
        println!("{:?}", parameters);
        if self.expect_next_token(TokenKind::LBRACE) {
            println!("TokenKind should be LBRACE but actually is {:?}",self.next_token.token_type)            
        }        

        let body = self.parse_block_statements(TokenKind::LBRACE)?;
        let expression = FunctionLiteral{
            parameters: Box::new(parameters),
            body: Box::new(body)
        };
        Ok(expression)
    }

    fn parse_function_parameters(&mut self) -> Result<Statement, Errors> {
        let mut statement: Vec<Statement> = vec![];
        if self.is_next_token(TokenKind::RPAREN) {
            self.next_token();
            return Ok(Statement::Parameter(statement))
        }
        self.next_token();
        statement.push(self.parse_statement()?);

        while self.is_next_token(TokenKind::COMMA) {
            self.next_token();
            self.next_token();
            statement.push(self.parse_statement()?);
        }
        if !self.expect_next_token(TokenKind::RPAREN) {
            panic!()
        }
        Ok(Statement::Parameter(statement))
    }


    fn parse_prefix_expression(&mut self) -> Result<PrefixExpression, Errors> {
        let current_token = self.current_token.literal.to_string();
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX)?;
        let expression = PrefixExpression{
                                           operator: current_token,
                                           right_expression: Box::new(right)
                                        };
        return Ok(expression)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, Errors> {
        let operator = match self.current_token.token_type {
            TokenKind::PLUS => "+".to_string(),
            TokenKind::MINUS => "-".to_string(),
            TokenKind::ASTERISK => "*".to_string(),
            TokenKind::SLASH => "/".to_string(),
            TokenKind::EQ => "==".to_string(),
            TokenKind::NotEq => "!=".to_string(),
            TokenKind::LT => "<".to_string(),
            TokenKind::GT => ">".to_string(),
            _ => {panic!()}
        };
        // the current token should be read in parse_expression().
        // so token must be read in order that the expression next operator
        // is set to current_token
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        let infix_expression = InfixExpression{
                                    left_expression: Box::new(left),
                                    operator: operator,
                                    right_expression: Box::new(right)
        };
        return Ok(Expression::InfixExpression(infix_expression))
    }

    fn current_precedence(&mut self) -> Precedence {
        return self.current_token.get_precedence()
    }

    fn next_precedence(&mut self) -> Precedence {
        return self.next_token.get_precedence()
    }

    fn is_current_token(&self, token_kind: TokenKind) -> bool {
        self.current_token.token_type == token_kind
    }

    fn is_next_token(&self, token_kind: TokenKind) -> bool {
        self.next_token.token_type == token_kind
    }

    fn expect_next_token(&mut self, token_kind: TokenKind) -> bool {
        let expect_token = self.is_next_token(token_kind);
        if expect_token {
            self.next_token();
        } else {
            println!("expect_token is {:?} but accually got {:?}", token_kind, self.next_token.token_type);
        }
        expect_token
    }
}

// if cfg(test) is written, test code is compiled only when test runs
#[cfg(test)]// test runs only when execute cargo run
mod testing {
    use crate::ast::ReturnStatement;
    use crate::lexer::Lexer;
    use crate::token::TokenKind;
    use crate::ast::Statement::Block;
    use crate::ast::Statement;
    use crate::ast::Expression;
    use crate::ast::IfExpression;
    use crate::ast::ExpressionStatement;
    use crate::ast::LetStatement;
    use crate::ast::Identifier;
    use crate::ast::Integer;
    use crate::ast::Bool;
    use crate::ast::PrefixExpression;
    use crate::ast::InfixExpression;
    use crate::parser::Parser;

    #[test]
    fn test_let_statement() {
        let input = r#"let x = 5;
                       let y = 10;
                       let foobar = 838383;"#;
        
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 3);

        let tests = vec![
            "x",
            "y",
            "foobar",
        ];

        for (i, test) in tests.iter().enumerate() {
            let stmt = &program.statements[i];
            assert_eq!(stmt, &Statement::LetStatement(LetStatement{identifier: Identifier{value: test.to_string()}}));
        }
    }

    #[test]
    fn test_return_statement() {
        let input = r#"return 5;
                       return 10;
                       return 993322;"#;
        
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 3);

        let tests = vec![
            "5",
            "10",
            "993322",
        ];

        for (i, test) in tests.iter().enumerate() {
            let stmt = &program.statements[i];
            assert_eq!(stmt, &Statement::ReturnStatement(ReturnStatement{identifier: Identifier{value: test.to_string()}}));
        }
    }
    #[test]
    fn test_identifier_expression() {
        let input = "foobar;".to_string();
        
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1); // 識別子が一つであること
        let test_ident = Statement::ExpressionStatement(ExpressionStatement{expression: Expression::Identifier(Identifier{value: "foobar".to_string()})});
        assert_eq!(program.statements[0], test_ident)
        }

        #[test]
        fn test_interger_expression() {
            let input = "5;".to_string();
            
            let lexer = Lexer::new(&input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
            let test_ident = Statement::ExpressionStatement(ExpressionStatement{expression: Expression::Integer(Integer{value: "5".to_string()})});
            assert_eq!(program.statements[0], test_ident)
            }

        #[test]
        fn test_prefix_expression() {
            let prefix_tests = vec![
                                ("!5", "!", 5),
                                ("-15", "-", 15)
                                   ];
            // compare the result of parseing the first element of tuple
            // with second, third elements.
            for (test, left, right) in prefix_tests.iter() {
                let lexer = Lexer::new(test);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
                let test_prefix = Statement::ExpressionStatement(ExpressionStatement{expression: Expression::PrefixExpression(
                PrefixExpression{operator: left.to_string(), right_expression: Box::new(Expression::Integer(Integer{value: right.to_string()}))})});
                assert_eq!(program.statements[0], test_prefix);
                }
            }

            #[test]
            fn test_infix_expression() {
                let infix_tests = vec![
                                    ("5 + 5;", 5, "+", 5),
                                    ("5 - 5;", 5, "-", 5),
                                    ("5 * 5;", 5, "*", 5),
                                    ("5 / 5;", 5, "/", 5),
                                    ("5 > 5;", 5, ">", 5),
                                    ("5 < 5;", 5, "<", 5),
                                    ("5 == 5;", 5, "==", 5),
                                    ("5 != 5;", 5, "!=", 5),
                                    ];
                // compare the result of parseing the first element of tuple
                // with second, third elements.
                for (test, left, middle, right) in infix_tests.iter() {
                    let lexer = Lexer::new(test);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program().unwrap();
                    assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
                    let test_infix = Statement::ExpressionStatement(
                        ExpressionStatement{expression: Expression::InfixExpression(
                        InfixExpression{left_expression: Box::new(Expression::Integer(Integer{value: left.to_string()})),
                                        operator: middle.to_string(),
                                        right_expression: Box::new(Expression::Integer(Integer{value: right.to_string()}))
                                        }
                                    )
                                }
                            );
                    assert_eq!(program.statements[0], test_infix);
                    }
                }

            #[test]
            fn test_operator_precedence_parsing() {
                let infix_tests = vec![
//                                    ("-a *b", "((-a) * b)"),
//                                    ("!-a", "(!(-a))"),
//                                    ("a + b + c", "((a + b) + c)"),
//                                    ("a + b - c", "((a + b) - c)"),
//                                    ("a * b * c", "((a * b) * c)"),
//                                    ("a * b / c", "((a * b) / c)"),
//                                    ("a + b / c", "((a + (b / c)"),
                                    ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4"),
                                    ("(5 + 5) * 2", "((5 + 5) * 2)"),
                                      ("2 / (5 + 5)", "(2 / (5 + 5))"),
                                    ("-(5 + 5)", "(-(5 + 5))"),
                                    ("!(true == true)", "(!(true == true))"),
                                    ];
                // compare the result of parseing the first element of tuple
                // with second, third elements.
                for (test, before) in infix_tests.iter() {
                    let lexer = Lexer::new(test);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program().unwrap();
                    println!("{:?}", program.statements[0]);
//                    assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
//                    eprint!("{:?}", program.statements[0]);
                }
                }
                #[test]
                fn test_bool_expression() {
                    let bool_tests = vec![
                                        ("true", true),
                                        ("false", false),
//                                        ("3 > 5 == false", "((3 > 5) == false)"),
//                                        ("3 < 5 == true", "((3 < 5) == true)"),
                                        ];
                    // compare the result of parseing the first element of tuple
                    // with second, third elements.
                    for (test, right) in bool_tests.iter() {
                        let lexer = Lexer::new(test);
                        let mut parser = Parser::new(lexer);
                        let program = parser.parse_program().unwrap();
//                        println!("{:?}", program);
                        assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
                        let test_bool = Statement::ExpressionStatement(
                            ExpressionStatement{
                                expression: Expression::Bool
                                    (Bool{value: *right
                                         }
                                    )
                                }
                            );
                        assert_eq!(program.statements[0], test_bool);
                        }
                    }    
                    #[test]
                    fn test_bool_infix_expression() {
                        let bool_tests = vec![
                                            ("3 > 5 == false", 3, ">", 5, "==", "false"),
                                            ("3 > 5 == false", 3, "<", 5, "==", "true"),
                                            ];
                        // compare the result of parseing the first element of tuple
                        // with second, third elements.
                        for (test, left, operator, right, bool_ident, bool_literal) in bool_tests.iter() {
                            let lexer = Lexer::new(test);
                            let mut parser = Parser::new(lexer);
                            let program = parser.parse_program();
//                            println!("{:?}", program.statements[0]);
//                            assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
//                            let test_bool = Statement::ExpressionStatement(
//                                ExpressionStatement{expression: Expression::InfixExpression(
//                                InfixExpression{left_expression: Box::new(Expression::Integer(Integer{value: left.to_string()})),
//                                                operator: operator.to_string(),
//                                                right_expression: Box::new(Expression::Integer(Integer{value: right.to_string()}))
//                                                }
//                                            ),
//                            
//                                        }
//                                    );        
//                            assert_eq!(program.statements[0], test_bool);
                            }
                        }    
            #[test]
            fn test_if_expression() {
                let input = "if (x < y) {x} else {y}".to_string();
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                println!("{:?}", program.statements[0]);
                }

            #[test]
            fn test_function_expression() {
                let input = "fn (x, y) {x + y;}".to_string();
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                println!("{:?}", program.statements[0]);
                }

            }