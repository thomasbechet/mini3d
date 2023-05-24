use crate::script::frontend::ast::BinaryOperator;

use super::{lexer::{Lexer, Token, TokenKind}, ast::{AST, ASTNodeId, ASTNode, Literal, ASTPrimitive}, error::{ParserError, LexerError}};

pub struct Parser<'s: 'a, 'a> {
    source: &'s str,
    lexer: Lexer<'s>,
    peeks: [Token; Parser::MAX_LOOKAHEAD],
    ast: &'a mut AST<'s>,
    parse_comments: bool,
}

impl<'s: 'a, 'a> Parser<'s, 'a> {

    const MAX_LOOKAHEAD: usize = 2;

    /// Translate peeked tokens
    fn advance(&mut self) -> Result<(), ParserError> {
        for i in (1..Self::MAX_LOOKAHEAD).rev() {
            self.peeks[i] = self.peeks[i - 1];
        }
        self.peeks[0] = self.lexer.next_token().map_err(ParserError::Lexer)?;
        Ok(())
    }

    /// Peek next token without consuming it
    /// 
    /// # Arguments
    /// 
    /// * `lookahead` - Number of tokens to look ahead
    fn peek(&mut self, lookahead: usize) -> Token {
        self.peeks[Self::MAX_LOOKAHEAD - lookahead - 1]
    }

    /// Consume next token
    fn consume(&mut self) -> Result<Token, ParserError> {
        let token = self.peek(0);
        self.advance()?;
        Ok(token)
    }

    /// Expect next token to be of a specific kind and consume it
    /// 
    /// # Arguments
    /// 
    /// * `kind` - Expected token kind
    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        let token = self.consume()?;
        if token.kind != kind {
            Err(ParserError::UnexpectedToken { expected: kind, got: token.kind })
        } else {
            Ok(token)
        }
    }

    /// Consume next token if it is of a specific kind or None otherwise
    /// 
    /// # Arguments
    /// 
    /// * `kind` - Expected token kind
    fn accept(&mut self, kind: TokenKind) -> Result<Option<Token>, ParserError> {
        let token = self.peek(0);
        if token.kind == kind {
            self.advance()?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn parse_member_lookup(&mut self, child: ASTNodeId) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::Dot)?;
        let ident = self.expect(TokenKind::Identifier)?;
        let node = self.ast.add(ASTNode::MemberLookup(ident.span.slice(self.source)));
        self.ast.append_child(node, child);
        if self.peek(0).kind == TokenKind::Dot {
            self.parse_member_lookup(node)
        } else {
            Ok(node)
        }
    }

    fn parse_identifier(&mut self) -> Result<ASTNodeId, ParserError> {
        let ident = self.expect(TokenKind::Identifier)?;
        let mut node = self.ast.add(ASTNode::Identifier(ident.span.slice(self.source)));
        if self.peek(0).kind == TokenKind::Dot {
            node = self.parse_member_lookup(node)?;
        }
        Ok(node)
    }

    fn parse_atom(&mut self) -> Result<ASTNodeId, ParserError> {
        let next = self.peek(0);
        let node = match next.kind {
            TokenKind::Identifier => {
                let ident = self.parse_identifier()?;
                if self.peek(0).kind == TokenKind::LeftParen {
                    self.parse_call(ident)?
                } else {
                    ident
                }
            },
            TokenKind::Integer => {
                self.consume()?;
                let value = next.span.slice(self.source).parse().map_err(ParserError::IntegerParseError)?;
                self.ast.add(ASTNode::Literal(Literal::Integer(value)))
            },
            TokenKind::Float => {
                self.consume()?;
                let value = next.span.slice(self.source).parse().map_err(ParserError::FloatParseError)?;
                self.ast.add(ASTNode::Literal(Literal::Float(value)))
            },
            TokenKind::String => {
                self.consume()?;
                let slice = next.span.string_content_slice(self.source);
                self.ast.add(ASTNode::Literal(Literal::String(slice)))
            } 
            TokenKind::True => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::Boolean(true)))
            },
            TokenKind::False => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::Boolean(false)))
            },
            TokenKind::Nil => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::Nil))
            },
            _ => return Err(ParserError::InvalidAtomExpression { got: next.kind })
        };
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<ASTNodeId, ParserError> {
        let token = self.peek(0);
        if token.kind == TokenKind::LeftParen {
            self.consume()?;
            let node = self.parse_expression(1)?;
            self.expect(TokenKind::RightParen)?;
            Ok(node)
        } else if token.kind.is_unaop() {
            self.consume()?;
            let expr = self.parse_primary()?;
            let node = self.ast.add(ASTNode::UnaryOperator(token.kind.into()));
            self.ast.append_child(node, expr);
            Ok(node)
        } else if token.kind.is_binop() {
            Err(ParserError::UnexpectedBinaryOperator)
        } else {
            Ok(self.parse_atom()?)
        }
    }

    fn parse_expression(&mut self, min_precedence: u32) -> Result<ASTNodeId, ParserError> {
        let mut lhs = self.parse_primary()?;
        loop {
            let current = self.peek(0);

            if current.kind.is_binop() {

                let op: BinaryOperator = current.kind.into();
                let prec = op.precedence();
                let left_assoc = op.is_left_associative();

                if prec < min_precedence { break; }
                let next_min_assoc = if left_assoc { prec + 1 } else { prec };

                self.consume()?;
                let rhs = self.parse_expression(next_min_assoc)?;

                let op_node = self.ast.add(ASTNode::BinaryOperator(op));
                self.ast.append_child(op_node, lhs);
                self.ast.append_child(op_node, rhs);
                lhs = op_node;

            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_statement(&mut self) -> Result<Option<ASTNodeId>, ParserError> {
        let next = self.peek(0);
        if next.kind == TokenKind::Let {
            Ok(Some(self.parse_variable_declaration()?))
        } else if next.kind == TokenKind::Function {
            Ok(Some(self.parse_function_declaration()?))
        } else if next.kind == TokenKind::Return {
            Ok(Some(self.parse_return_statement()?))
        } else if next.kind == TokenKind::If {
            Ok(Some(self.parse_if_statement()?))
        } else if next.kind == TokenKind::Identifier {
            let ident = self.parse_identifier()?;
            if self.peek(0).kind == TokenKind::Assign {
                Ok(Some(self.parse_assignment_statement(ident)?))
            } else if self.peek(0).kind == TokenKind::LeftParen {
                Ok(Some(self.parse_call(ident)?))
            } else {
                Err(ParserError::IdentifierAsStatement)
            }
        } else if next.kind == TokenKind::Comment {
            let comment = self.consume()?;
            if self.parse_comments {
                Ok(Some(self.ast.add(ASTNode::CommentStatement(comment.span.comment_content_slice(self.source)))))
            } else {
                self.parse_statement() // Ignore comments
            }
        } else if next.kind == TokenKind::Import {
            Err(ParserError::UnexpectedImportStatement)
        } else {
            Ok(None)
        }
    }

    fn try_parse_primitive_type(&mut self) -> Result<Option<ASTPrimitive>, ParserError> {
        if self.accept(TokenKind::Colon)?.is_some() {
            let ident = self.expect(TokenKind::Identifier)?.span.slice(self.source);
            Ok(ASTPrimitive::parse(ident))
        } else {
            Ok(None)
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::Let)?;
        let ident = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0)?;
        let node = self.ast.add(ASTNode::VariableDeclaration { identifier: ident.span.slice(self.source), var_type: primitive });
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_function_argument(&mut self) -> Result<ASTNodeId, ParserError> {
        let ident = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        Ok(self.ast.add(ASTNode::FunctionArgument { identifier: ident.span.slice(self.source), arg_type: primitive }))
    }

    fn parse_function_declaration(&mut self) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::Function)?;
        // Identifier
        let ident = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::LeftParen)?;
        let function = self.ast.add(ASTNode::FunctionDeclaration {
            identifier: ident.span.slice(self.source),
            return_type: None,
        });
        // First argument
        if self.peek(0).kind != TokenKind::RightParen {
            let arg = self.parse_function_argument()?;
            self.ast.append_child(function, arg);
        }
        // Rest of the arguments
        while self.accept(TokenKind::Comma)?.is_some() {
            let arg = self.parse_function_argument()?;
            self.ast.append_child(function, arg);
        }
        self.expect(TokenKind::RightParen)?;
        if let Some(primitive) = self.try_parse_primitive_type()? {
            // Update return type
            if let ASTNode::FunctionDeclaration { return_type, .. }  = self.ast.get_mut(function).unwrap() {
                *return_type = Some(primitive);
            }
        }
        
        while let Some(stmt) = self.parse_statement()? {
            self.ast.append_child(function, stmt);
        }
        self.expect(TokenKind::End)?;
        Ok(function)
    }

    fn parse_return_statement(&mut self) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::Return)?;
        let expr = self.parse_expression(0)?;
        let node = self.ast.add(ASTNode::ReturnStatement);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_and_append_if_body(&mut self, if_node: ASTNodeId) -> Result<(), ParserError> {
        let body = self.ast.add(ASTNode::IfBody);
        while let Some(stmt) = self.parse_statement()? {
            self.ast.append_child(body, stmt);
        }
        self.ast.append_child(if_node, body);
        Ok(())
    }

    fn parse_if_statement(&mut self) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::If)?;
        let node = self.ast.add(ASTNode::IfStatement);
        // If condition
        let condition = self.parse_expression(0)?;
        self.ast.append_child(node, condition);
        self.expect(TokenKind::Then)?;
        // If body
        self.parse_and_append_if_body(node)?;
        while self.accept(TokenKind::Elif)?.is_some() {
            // Elif condition
            let condition = self.parse_expression(0)?;
            self.ast.append_child(node, condition);
            self.expect(TokenKind::Then)?;
            // Elif body
            self.parse_and_append_if_body(node)?;
        }
        if self.accept(TokenKind::Else)?.is_some() {
            // Else body
            self.parse_and_append_if_body(node)?;
        }
        self.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_for_statement(&mut self) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::For)?;
        let ident = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::In)?;
        let expr = self.parse_expression(0)?;

        let node = self.ast.add(ASTNode::ForStatement);
        // self.ast.append_child(node, )?;
        self.ast.append_child(node, expr);
        self.expect(TokenKind::Do)?;
        while let Some(stmt) = self.parse_statement()? {
            self.ast.append_child(node, stmt);
        }
        self.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_assignment_statement(&mut self, ident: ASTNodeId) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0)?;
        let node = self.ast.add(ASTNode::Assignment);
        self.ast.append_child(node, ident);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_call(&mut self, ident: ASTNodeId) -> Result<ASTNodeId, ParserError> {
        let node = self.ast.add(ASTNode::Call);
        self.ast.append_child(node, ident);
        self.expect(TokenKind::LeftParen)?;
        if self.peek(0).kind != TokenKind::RightParen {
            let expr = self.parse_expression(0)?;
            self.ast.append_child(node, expr);
        }
        while self.accept(TokenKind::Comma)?.is_some() {
            let expr = self.parse_expression(0)?;
            self.ast.append_child(node, expr);
        }
        self.expect(TokenKind::RightParen)?;
        Ok(node)
    }

    fn parse_import(&mut self) -> Result<ASTNodeId, ParserError> {
        self.expect(TokenKind::Import)?;
        let path = self.expect(TokenKind::String)?;
        self.expect(TokenKind::As)?;
        let ident = self.expect(TokenKind::Identifier)?;
        let node = self.ast.add(ASTNode::Import { path: path.span.string_content_slice(self.source), identifier: ident.span.slice(self.source) });
        Ok(node)
    }

    fn build_ast(&mut self) -> Result<(), ParserError> {

        while self.peek(0).kind == TokenKind::Import {
            let import = self.parse_import()?;
            self.ast.append_child(self.ast.root(), import);
        }

        while let Some(stmt) = self.parse_statement()? {
            self.ast.append_child(self.ast.root(), stmt);
        }

        Ok(())
    }

    pub fn parse(source: &'s str, parse_comments: bool) -> Result<AST<'s>, ParserError> {
        let mut ast = AST::new();
        let mut parser = Parser {
            source,
            lexer: Lexer::new(source),
            peeks: [Token::eof(); Parser::MAX_LOOKAHEAD],
            ast: &mut ast,
            parse_comments,
        };
        for _ in 0..Parser::MAX_LOOKAHEAD {
            parser.advance()?;
        }
        parser.build_ast()?;
        Ok(ast)
    }
}


