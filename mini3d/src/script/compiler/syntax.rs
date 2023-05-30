use crate::script::compiler::ast::BinaryOperator;

use super::{
    ast::{ASTNode, ASTNodeId, Literal, AST},
    error::{CompilationError, SyntaxError},
    lexical::Lexer,
    symbol::{BlockId, PrimitiveType, SymbolKind, SymbolTable},
    token::{Token, TokenKind},
};

pub(crate) struct Parser<'s: 'a, 'a> {
    source: &'s str,
    lexer: Lexer<'s>,
    peeks: [Token; Parser::MAX_LOOKAHEAD],
    ast: &'a mut AST,
    symbols: &'a mut SymbolTable,
    parse_comments: bool,
}

impl<'s: 'a, 'a> Parser<'s, 'a> {
    const MAX_LOOKAHEAD: usize = 2;

    /// Translate peeked tokens
    fn advance(&mut self) -> Result<(), CompilationError> {
        for i in (1..Self::MAX_LOOKAHEAD).rev() {
            self.peeks[i] = self.peeks[i - 1];
        }
        self.peeks[0] = self.lexer.next_token()?;
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
    fn consume(&mut self) -> Result<Token, CompilationError> {
        let token = self.peek(0);
        self.advance()?;
        Ok(token)
    }

    /// Expect next token to be of a specific kind and consume it
    ///
    /// # Arguments
    ///
    /// * `kind` - Expected token kind
    fn expect(&mut self, kind: TokenKind) -> Result<Token, CompilationError> {
        let token = self.consume()?;
        if token.kind != kind {
            Err(SyntaxError::UnexpectedToken {
                span: token.span,
                expected: kind,
                got: token.kind,
            }
            .into())
        } else {
            Ok(token)
        }
    }

    /// Consume next token if it is of a specific kind or None otherwise
    ///
    /// # Arguments
    ///
    /// * `kind` - Expected token kind
    fn accept(&mut self, kind: TokenKind) -> Result<Option<Token>, CompilationError> {
        let token = self.peek(0);
        if token.kind == kind {
            self.advance()?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn parse_member_lookup(&mut self, child: ASTNodeId) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Dot)?;
        let token = self.expect(TokenKind::Identifier)?;
        let node = self.ast.add(ASTNode::MemberLookup { span: token.span });
        self.ast.append_child(node, child);
        if self.peek(0).kind == TokenKind::Dot {
            self.parse_member_lookup(node)
        } else {
            Ok(node)
        }
    }

    fn parse_identifier(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        let token = self.expect(TokenKind::Identifier)?;
        let ident = token.span.slice(self.source);
        // Find symbol or mark as unresolved
        let symbol = self.symbols.lookup(ident, block);
        let mut node = self.ast.add(ASTNode::Identifier {
            span: token.span,
            symbol,
        });
        if self.peek(0).kind == TokenKind::Dot {
            node = self.parse_member_lookup(node)?;
        }
        Ok(node)
    }

    fn parse_atom(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        let next = self.peek(0);
        let node = match next.kind {
            TokenKind::Identifier => {
                let ident = self.parse_identifier(block)?;
                if self.peek(0).kind == TokenKind::LeftParen {
                    self.parse_call(ident, block)?
                } else {
                    ident
                }
            }
            TokenKind::Integer => {
                self.consume()?;
                let value = next.span.slice(self.source).parse().map_err(|error| {
                    SyntaxError::IntegerParseError {
                        span: next.span,
                        error,
                    }
                })?;
                self.ast.add(ASTNode::Literal(Literal::Integer(value)))
            }
            TokenKind::Float => {
                self.consume()?;
                let value = next.span.slice(self.source).parse().map_err(|error| {
                    SyntaxError::FloatParseError {
                        span: next.span,
                        error,
                    }
                })?;
                self.ast.add(ASTNode::Literal(Literal::Float(value)))
            }
            TokenKind::String => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::String(next.span)))
            }
            TokenKind::True => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::Boolean(false)))
            }
            TokenKind::Nil => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(Literal::Nil))
            }
            _ => {
                return Err(SyntaxError::InvalidAtomExpression {
                    got: next.kind,
                    span: next.span,
                }
                .into())
            }
        };
        Ok(node)
    }

    fn parse_primary(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        let token = self.peek(0);
        if token.kind == TokenKind::LeftParen {
            self.consume()?;
            let node = self.parse_expression(1, block)?;
            self.expect(TokenKind::RightParen)?;
            Ok(node)
        } else if token.kind.is_unaop() {
            self.consume()?;
            let expr = self.parse_primary(block)?;
            let node = self.ast.add(ASTNode::UnaryOperator(token.kind.into()));
            self.ast.append_child(node, expr);
            Ok(node)
        } else if token.kind.is_binop() {
            Err(SyntaxError::UnexpectedBinaryOperator { span: token.span }.into())
        } else {
            Ok(self.parse_atom(block)?)
        }
    }

    fn parse_expression(
        &mut self,
        min_precedence: u32,
        block: BlockId,
    ) -> Result<ASTNodeId, CompilationError> {
        let mut lhs = self.parse_primary(block)?;
        loop {
            let current = self.peek(0);

            if current.kind.is_binop() {
                let op: BinaryOperator = current.kind.into();
                let prec = op.precedence();
                let left_assoc = op.is_left_associative();

                if prec < min_precedence {
                    break;
                }
                let next_min_assoc = if left_assoc { prec + 1 } else { prec };

                self.consume()?;
                let rhs = self.parse_expression(next_min_assoc, block)?;

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

    fn parse_statement(&mut self, block: BlockId) -> Result<Option<ASTNodeId>, CompilationError> {
        let next = self.peek(0);
        if next.kind == TokenKind::Let {
            Ok(Some(self.parse_variable_declaration(block)?))
        } else if next.kind == TokenKind::Const {
            Ok(Some(self.parse_constant_declaration(block)?))
        } else if next.kind == TokenKind::Function {
            Ok(Some(self.parse_function_declaration(block)?))
        } else if next.kind == TokenKind::Return {
            Ok(Some(self.parse_return_statement(block)?))
        } else if next.kind == TokenKind::If {
            Ok(Some(self.parse_if_statement(block)?))
        } else if next.kind == TokenKind::Identifier {
            let ident = self.parse_identifier(block)?;
            if self.peek(0).kind == TokenKind::Assign {
                Ok(Some(self.parse_assignment_statement(ident, block)?))
            } else if self.peek(0).kind == TokenKind::LeftParen {
                Ok(Some(self.parse_call(ident, block)?))
            } else {
                Err(SyntaxError::IdentifierAsStatement { span: next.span }.into())
            }
        } else if next.kind == TokenKind::Comment {
            let comment = self.consume()?;
            if self.parse_comments {
                Ok(Some(
                    self.ast
                        .add(ASTNode::CommentStatement { span: comment.span }),
                ))
            } else {
                self.parse_statement(block) // Ignore comments
            }
        } else if next.kind == TokenKind::Import {
            Err(SyntaxError::UnexpectedImportStatement { span: next.span }.into())
        } else {
            Ok(None)
        }
    }

    fn try_parse_primitive_type(&mut self) -> Result<Option<PrimitiveType>, CompilationError> {
        if self.accept(TokenKind::Colon)?.is_some() {
            let ident = self.expect(TokenKind::Identifier)?.span.slice(self.source);
            Ok(PrimitiveType::parse(ident))
        } else {
            Ok(None)
        }
    }

    fn parse_variable_declaration(
        &mut self,
        block: BlockId,
    ) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Let)?;
        let token = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = self.symbols.declare_symbol(
            token.span.slice(self.source),
            SymbolKind::Variable {
                var_type: primitive,
            },
            block,
        );
        let node = self.ast.add(ASTNode::VariableDeclaration {
            span: token.span,
            symbol,
        });
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_constant_declaration(
        &mut self,
        block: BlockId,
    ) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Const)?;
        let token = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = self.symbols.declare_symbol(
            token.span.slice(self.source),
            SymbolKind::Constant {
                const_type: primitive,
            },
            block,
        );
        let node = self.ast.add(ASTNode::ConstantDeclaration {
            span: token.span,
            symbol,
        });
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_function_argument(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        let token = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        let ident = token.span.slice(self.source);
        // Check duplicated argument
        if self.symbols.find_in_block(ident, block).is_some() {
            return Err(SyntaxError::DuplicatedArgument { span: token.span }.into());
        }
        let symbol = self.symbols.declare_symbol(
            ident,
            SymbolKind::Variable {
                var_type: primitive,
            },
            block,
        );
        Ok(self.ast.add(ASTNode::FunctionArgument {
            span: token.span,
            symbol,
        }))
    }

    fn parse_function_declaration(
        &mut self,
        block: BlockId,
    ) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Function)?;
        // Identifier
        let token = self.expect(TokenKind::Identifier)?;
        let slice = token.span.slice(self.source);
        let symbol =
            self.symbols
                .declare_symbol(slice, SymbolKind::Function { return_type: None }, block);
        let function_block = self.symbols.define_block(Some(block));
        let function = self.ast.add(ASTNode::FunctionDeclaration {
            span: token.span,
            symbol,
            function_block,
        });
        self.expect(TokenKind::LeftParen)?;
        // First argument
        if self.peek(0).kind != TokenKind::RightParen {
            let arg = self.parse_function_argument(function_block)?;
            self.ast.append_child(function, arg);
        }
        // Rest of the arguments
        while self.accept(TokenKind::Comma)?.is_some() {
            let arg = self.parse_function_argument(function_block)?;
            self.ast.append_child(function, arg);
        }
        self.expect(TokenKind::RightParen)?;
        if let Some(primitive) = self.try_parse_primitive_type()? {
            // Update return type
            if let ASTNode::FunctionDeclaration { symbol, .. } = self.ast.get_mut(function).unwrap()
            {
                match &mut self.symbols.get_mut(*symbol).kind {
                    SymbolKind::Function { return_type, .. } => {
                        *return_type = Some(primitive);
                    }
                    _ => unreachable!(),
                }
            }
        }
        while let Some(stmt) = self.parse_statement(function_block)? {
            self.ast.append_child(function, stmt);
        }
        self.expect(TokenKind::End)?;
        Ok(function)
    }

    fn parse_return_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Return)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::ReturnStatement);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_and_append_if_body(
        &mut self,
        if_node: ASTNodeId,
        block: BlockId,
    ) -> Result<(), CompilationError> {
        let body = self.ast.add(ASTNode::IfBody);
        while let Some(stmt) = self.parse_statement(block)? {
            self.ast.append_child(body, stmt);
        }
        self.ast.append_child(if_node, body);
        Ok(())
    }

    fn parse_if_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::If)?;
        let node = self.ast.add(ASTNode::IfStatement);
        // If condition
        let condition = self.parse_expression(0, block)?;
        self.ast.append_child(node, condition);
        self.expect(TokenKind::Then)?;
        // If body
        let if_block = self.symbols.define_block(Some(block));
        self.parse_and_append_if_body(node, if_block)?;
        while self.accept(TokenKind::Elif)?.is_some() {
            // Elif condition
            let condition = self.parse_expression(0, block)?;
            self.ast.append_child(node, condition);
            self.expect(TokenKind::Then)?;
            // Elif body
            self.parse_and_append_if_body(node, if_block)?;
        }
        if self.accept(TokenKind::Else)?.is_some() {
            // Else body
            self.parse_and_append_if_body(node, block)?;
        }
        self.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_for_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::For)?;
        let for_block = self.symbols.define_block(Some(block));
        let token = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::In)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::ForStatement);
        // self.ast.append_child(node, )?;
        self.ast.append_child(node, expr);
        self.expect(TokenKind::Do)?;
        while let Some(stmt) = self.parse_statement(for_block)? {
            self.ast.append_child(node, stmt);
        }
        self.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_assignment_statement(
        &mut self,
        ident: ASTNodeId,
        block: BlockId,
    ) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::Assignment);
        self.ast.append_child(node, ident);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_call(
        &mut self,
        ident: ASTNodeId,
        block: BlockId,
    ) -> Result<ASTNodeId, CompilationError> {
        let node = self.ast.add(ASTNode::Call);
        self.ast.append_child(node, ident);
        self.expect(TokenKind::LeftParen)?;
        if self.peek(0).kind != TokenKind::RightParen {
            let expr = self.parse_expression(0, block)?;
            self.ast.append_child(node, expr);
        }
        while self.accept(TokenKind::Comma)?.is_some() {
            let expr = self.parse_expression(0, block)?;
            self.ast.append_child(node, expr);
        }
        self.expect(TokenKind::RightParen)?;
        Ok(node)
    }

    fn parse_import(&mut self, block: BlockId) -> Result<ASTNodeId, CompilationError> {
        self.expect(TokenKind::Import)?;
        let path = self.expect(TokenKind::String)?;
        self.expect(TokenKind::As)?;
        let token = self.expect(TokenKind::Identifier)?;
        let ident = token.span.slice(self.source);
        let symbol = self
            .symbols
            .declare_symbol(ident, SymbolKind::Import, block);
        let node = self.ast.add(ASTNode::Import {
            path: token.span,
            symbol,
        });
        Ok(node)
    }

    fn parse(&mut self) -> Result<(), CompilationError> {
        for _ in 0..Parser::MAX_LOOKAHEAD {
            self.advance()?;
        }

        let global_block = self.symbols.define_block(None);

        while self.peek(0).kind == TokenKind::Import {
            let import = self.parse_import(global_block)?;
            self.ast.append_child(self.ast.root(), import);
        }

        while let Some(stmt) = self.parse_statement(global_block)? {
            self.ast.append_child(self.ast.root(), stmt);
        }

        Ok(())
    }
}

pub(crate) struct SyntaxAnalysis;

impl SyntaxAnalysis {
    pub(crate) fn evaluate(
        lexer: Lexer<'_>,
        parse_comments: bool,
    ) -> Result<(AST, SymbolTable), CompilationError> {
        let mut ast = AST::new();
        let mut symbols = SymbolTable::default();
        let mut parser = Parser {
            source: lexer.source,
            lexer,
            peeks: [Token::eof(); Parser::MAX_LOOKAHEAD],
            ast: &mut ast,
            symbols: &mut symbols,
            parse_comments,
        };
        parser.parse()?;
        Ok((ast, symbols))
    }
}
