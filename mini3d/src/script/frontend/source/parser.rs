use crate::script::{
    frontend::error::{CompileError, SyntaxError},
    mir::primitive::PrimitiveType,
};

use super::{
    ast::{ASTNode, ASTNodeId, AST},
    lexer::Lexer,
    operator::BinaryOperator,
    stream::SourceStream,
    strings::StringTable,
    symbol::{BlockId, Symbol, SymbolTable},
    token::{Location, Token, TokenKind},
};

pub(crate) struct Parser<'a, S: Iterator<Item = (char, Location)>> {
    ast: &'a mut AST,
    lexer: &'a mut Lexer,
    symbols: &'a mut SymbolTable,
    strings: &'a mut StringTable,
    source: &'a mut S,
}

impl<'a, S: Iterator<Item = (char, Location)>> Parser<'a, S> {
    fn peek(&mut self, n: usize) -> Result<Token, CompileError> {
        self.lexer.peek(self.source, self.strings, n)
    }

    fn consume(&mut self) -> Result<Token, CompileError> {
        self.lexer.next(self.source, self.strings)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, CompileError> {
        let token = self.consume()?;
        if token.kind != kind {
            Err(SyntaxError::UnexpectedToken {
                span: token.span,
                got: token.kind,
            }
            .into())
        } else {
            Ok(token)
        }
    }

    fn accept(&mut self, kind: TokenKind) -> Result<Option<Token>, CompileError> {
        let token = self.lexer.peek(self.source, self.strings, 0)?;
        if token.kind == kind {
            self.lexer.next(self.source, self.strings)?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn parse_member_lookup(&mut self, child: ASTNodeId) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::Dot)?;
        let token = self.expect(TokenKind::Identifier)?;
        let node = self.ast.add(ASTNode::MemberLookup {
            span: token.span,
            ident: token.value.into(),
        });
        self.ast.append_child(node, child);
        if self.peek(0)?.kind == TokenKind::Dot {
            self.parse_member_lookup(node)
        } else {
            Ok(node)
        }
    }

    fn parse_identifier(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let token = self.expect(TokenKind::Identifier)?;
        let symbol = self
            .symbols
            .lookup_symbol(self.strings, token.value.into(), block);
        let mut node = self.ast.add(ASTNode::Identifier {
            span: token.span,
            symbol,
        });
        if self.peek(0)?.kind == TokenKind::Dot {
            node = self.parse_member_lookup(node)?;
        }
        Ok(node)
    }

    fn parse_atom(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let next = self.peek(0)?;
        let node = match next.kind {
            TokenKind::Identifier => {
                let ident = self.parse_identifier(block)?;
                if self.peek(0)?.kind == TokenKind::LeftParen {
                    self.parse_call(ident, block)?
                } else {
                    ident
                }
            }
            TokenKind::Literal => {
                self.consume()?;
                self.ast.add(ASTNode::Literal(next.value.into()))
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

    fn parse_primary(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let token = self.peek(0)?;
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
    ) -> Result<ASTNodeId, CompileError> {
        let mut lhs = self.parse_primary(block)?;
        loop {
            let current = self.peek(0)?;

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

    fn parse_statement(
        &mut self,
        block: BlockId,
        end_token: TokenKind,
    ) -> Result<Option<ASTNodeId>, CompileError> {
        let next = self.peek(0)?;
        if next.kind == end_token {
            self.consume()?;
            return Ok(None);
        }
        if next.kind == TokenKind::Let {
            Ok(Some(self.parse_variable_declaration(block)?))
        } else if next.kind == TokenKind::Export {
            self.consume()?;
            let next = self.peek(0)?;
            if next.kind == TokenKind::Function {
                Ok(Some(self.parse_function_declaration(block)?))
            } else if next.kind == TokenKind::Const {
                Ok(Some(self.parse_constant_declaration(block, true)?))
            } else {
                Err(SyntaxError::UnexpectedToken {
                    span: next.span,
                    got: next.kind,
                }
                .into())
            }
        } else if next.kind == TokenKind::Const {
            Ok(Some(self.parse_constant_declaration(block, false)?))
        } else if next.kind == TokenKind::Function {
            Ok(Some(self.parse_function_declaration(block)?))
        } else if next.kind == TokenKind::Return {
            Ok(Some(self.parse_return_statement(block)?))
        } else if next.kind == TokenKind::If {
            Ok(Some(self.parse_if_statement(block)?))
        } else if next.kind == TokenKind::Identifier {
            let ident = self.parse_identifier(block)?;
            if self.peek(0)?.kind == TokenKind::Assign {
                Ok(Some(self.parse_assignment_statement(ident, block)?))
            } else if self.peek(0)?.kind == TokenKind::LeftParen {
                Ok(Some(self.parse_call(ident, block)?))
            } else {
                Err(SyntaxError::IdentifierAsStatement { span: next.span }.into())
            }
        } else if next.kind == TokenKind::Comment {
            let comment = self.consume()?;
            Ok(Some(self.ast.add(ASTNode::Comment {
                span: comment.span,
                value: comment.value.into(),
            })))
        } else {
            Err(SyntaxError::NonStatementToken {
                got: next.kind,
                span: next.span,
            }
            .into())
        }
    }

    fn try_parse_primitive_type(&mut self) -> Result<Option<PrimitiveType>, CompileError> {
        if self.accept(TokenKind::Colon)?.is_some() {
            Ok(Some(self.expect(TokenKind::PrimitiveType)?.value.into()))
        } else {
            Ok(None)
        }
    }

    fn parse_variable_declaration(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::Let)?;
        let token = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = self.symbols.define_symbol(
            self.strings,
            token.value.into(),
            block,
            token.span,
            Symbol::Variable {
                var_type: primitive,
            },
            true,
        )?;
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
        export: bool,
    ) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::Const)?;
        let token = self.expect(TokenKind::Identifier)?;
        let const_type = self
            .try_parse_primitive_type()?
            .ok_or(SyntaxError::MissingConstantType { span: token.span })?;
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = Symbol::Constant { value: None };
        let symbol_id = self.symbols.define_symbol(
            self.strings,
            token.value.into(),
            block,
            token.span,
            symbol,
            false,
        )?;
        if export && block != SymbolTable::GLOBAL_BLOCK {
            return Err(
                SyntaxError::ExportedConstantOutsideOfGlobalScope { span: token.span }.into(),
            );
        }
        let node = self.ast.add(ASTNode::ConstantDeclaration {
            span: token.span,
            symbol: symbol_id,
        });
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_function_declaration(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        if block != SymbolTable::GLOBAL_BLOCK {
            return Err(SyntaxError::FunctionDeclarationOutsideOfGlobalScope {
                span: self.peek(0).unwrap().span,
            }
            .into());
        }
        self.expect(TokenKind::Function)?;
        let token = self.expect(TokenKind::Identifier)?;
        let symbol_id = self.symbols.define_symbol(
            self.strings,
            token.value.into(),
            block,
            token.span,
            Symbol::Function {
                return_type: None,
                first_arg: None,
            },
            false,
        )?;
        let function_block = self.symbols.add_block(Some(block));
        let function = self.ast.add(ASTNode::FunctionDeclaration {
            span: token.span,
            symbol: symbol_id,
            function_block,
        });
        self.expect(TokenKind::LeftParen)?;
        // First argument
        if self.peek(0)?.kind != TokenKind::RightParen {
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
                if let Some(Symbol::Function { return_type, .. }) = self.symbols.get_mut(*symbol) {
                    *return_type = Some(primitive);
                }
            }
        }
        while let Some(stmt) = self.parse_statement(function_block, TokenKind::End)? {
            self.ast.append_child(function, stmt);
        }
        Ok(function)
    }

    fn parse_function_argument(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let token = self.expect(TokenKind::Identifier)?;
        let primitive = self.try_parse_primitive_type()?;
        // Check duplicated argument
        if self
            .symbols
            .find_in_block(self.strings, token.value.into(), block)
            .is_some()
        {
            return Err(SyntaxError::DuplicatedArgument { span: token.span }.into());
        }
        let symbol = self.symbols.define_symbol(
            self.strings,
            token.value.into(),
            block,
            token.span,
            Symbol::Variable {
                var_type: primitive,
            },
            true,
        )?;
        Ok(self.ast.add(ASTNode::FunctionArgument {
            span: token.span,
            symbol,
        }))
    }

    fn parse_return_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::Return)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::Return);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_and_append_if_body(
        &mut self,
        if_node: ASTNodeId,
        block: BlockId,
    ) -> Result<(), CompileError> {
        let body = self.ast.add(ASTNode::IfBody);
        while let Some(stmt) = self.parse_statement(block, TokenKind::End)? {
            self.ast.append_child(body, stmt);
        }
        self.ast.append_child(if_node, body);
        Ok(())
    }

    fn parse_if_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::If)?;
        let node = self.ast.add(ASTNode::If);
        // If condition
        let condition = self.parse_expression(0, block)?;
        self.ast.append_child(node, condition);
        self.expect(TokenKind::Then)?;
        // If body
        let if_block = self.symbols.add_block(Some(block));
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

    fn parse_for_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::For)?;
        let for_block = self.symbols.add_block(Some(block));
        let token = self.expect(TokenKind::Identifier)?;
        self.expect(TokenKind::In)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::For);
        // self.ast.append_child(node, )?;
        self.ast.append_child(node, expr);
        self.expect(TokenKind::Do)?;
        while let Some(stmt) = self.parse_statement(for_block, TokenKind::End)? {
            self.ast.append_child(node, stmt);
        }
        self.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_assignment_statement(
        &mut self,
        ident: ASTNodeId,
        block: BlockId,
    ) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::Assignment);
        self.ast.append_child(node, ident);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_call(&mut self, ident: ASTNodeId, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let node = self.ast.add(ASTNode::Call);
        self.ast.append_child(node, ident);
        self.expect(TokenKind::LeftParen)?;
        if self.peek(0)?.kind != TokenKind::RightParen {
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

    fn parse_import(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.expect(TokenKind::Import)?;
        let path = self.expect(TokenKind::Literal)?;
        self.expect(TokenKind::As)?;
        let token = self.expect(TokenKind::Identifier)?;
        let symbol = self.symbols.define_symbol(
            self.strings,
            token.value.into(),
            block,
            token.span,
            Symbol::Module {
                path: token.value.into(),
            },
            false,
        )?;
        let node = self.ast.add(ASTNode::Import {
            path: token.span,
            symbol,
        });
        Ok(node)
    }

    fn parse(&mut self) -> Result<(), CompileError> {
        let global_block = self.symbols.add_block(None);

        // while self.peek(0)?.kind == TokenKind::Import {
        //     let import = self.parse_import(global_block)?;
        //     self.ast.append_child(self.ast.root(), import);
        // }

        while let Some(stmt) = self.parse_statement(global_block, TokenKind::EOF)? {
            self.ast.append_child(self.ast.root(), stmt);
        }

        Ok(())
    }

    pub(crate) fn evaluate(
        ast: &mut AST,
        symbols: &mut SymbolTable,
        strings: &mut StringTable,
        lexer: &mut Lexer,
        source: &mut SourceStream,
    ) -> Result<(), CompileError> {
        let mut parser = Parser {
            lexer,
            ast,
            symbols,
            strings,
            source,
        };
        parser.parse()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let mut strings = StringTable::default();
        let mut symbols = SymbolTable::default();
        let mut source = SourceStream::new(
            r#"
        let x = 1 + 2 * 3
        function y(a, b)
            return a + b
        end
        "#,
        );
        let mut ast = AST::default();
        Parser::<SourceStream>::evaluate(
            &mut ast,
            &mut symbols,
            &mut strings,
            &mut Lexer::new(false),
            &mut source,
        )
        .unwrap();
        ast.print();
    }
}
