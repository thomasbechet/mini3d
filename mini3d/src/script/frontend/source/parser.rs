use crate::{
    script::{
        compiler::CompilationUnit,
        export::{Export, ExportTable},
        frontend::error::{CompileError, SyntaxError},
        mir::primitive::PrimitiveType,
        module::ModuleId,
    },
    uid::UID,
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
    lexer: &'a mut Lexer,
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
                expect: kind,
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

    fn try_parse_primitive_type(&mut self) -> Result<Option<PrimitiveType>, CompileError> {
        if self.accept(TokenKind::Colon)?.is_some() {
            Ok(Some(self.expect(TokenKind::PrimitiveType)?.value.into()))
        } else {
            Ok(None)
        }
    }
}

pub(crate) struct ASTParser<'a, S: Iterator<Item = (char, Location)>> {
    parser: &'a mut Parser<'a, S>,
    ast: &'a mut AST,
    symbols: &'a mut SymbolTable,
}

impl<'a, S: Iterator<Item = (char, Location)>> ASTParser<'a, S> {
    fn parse_member_lookup(&mut self, child: ASTNodeId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Dot)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let node = self.ast.add(ASTNode::MemberLookup {
            span: token.span,
            ident: token.value.into(),
        });
        self.ast.append_child(node, child);
        if self.parser.peek(0)?.kind == TokenKind::Dot {
            self.parse_member_lookup(node)
        } else {
            Ok(node)
        }
    }

    fn parse_identifier(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let token = self.parser.expect(TokenKind::Identifier)?;
        let symbol = self
            .symbols
            .lookup_symbol(self.parser.strings, token.value.into(), block);
        let mut node = self.ast.add(ASTNode::Identifier {
            span: token.span,
            symbol,
        });
        if self.parser.peek(0)?.kind == TokenKind::Dot {
            node = self.parse_member_lookup(node)?;
        }
        Ok(node)
    }

    fn parse_atom(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let next = self.parser.peek(0)?;
        let node = match next.kind {
            TokenKind::Identifier => {
                let ident = self.parse_identifier(block)?;
                if self.parser.peek(0)?.kind == TokenKind::LeftParen {
                    self.parse_call(ident, block)?
                } else {
                    ident
                }
            }
            TokenKind::Literal => {
                self.parser.consume()?;
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
        let token = self.parser.peek(0)?;
        if token.kind == TokenKind::LeftParen {
            self.parser.consume()?;
            let node = self.parse_expression(1, block)?;
            self.parser.expect(TokenKind::RightParen)?;
            Ok(node)
        } else if token.kind.is_unaop() {
            self.parser.consume()?;
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
            let current = self.parser.peek(0)?;

            if current.kind.is_binop() {
                let op: BinaryOperator = current.kind.into();
                let prec = op.precedence();
                let left_assoc = op.is_left_associative();

                if prec < min_precedence {
                    break;
                }
                let next_min_assoc = if left_assoc { prec + 1 } else { prec };

                self.parser.consume()?;
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

    fn parse_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let next = self.parser.peek(0)?;
        if next.kind == TokenKind::Let {
            Ok(self.parse_variable_declaration(block)?)
        } else if next.kind == TokenKind::Export {
            self.parser.consume()?;
            let next = self.parser.peek(0)?;
            if next.kind == TokenKind::Function {
                Ok(self.parse_function_declaration(block)?)
            } else if next.kind == TokenKind::Const {
                Ok(self.parse_constant_declaration(block, true)?)
            } else {
                Err(SyntaxError::UnexpectedExportToken {
                    span: next.span,
                    got: next.kind,
                }
                .into())
            }
        } else if next.kind == TokenKind::Const {
            Ok(self.parse_constant_declaration(block, false)?)
        } else if next.kind == TokenKind::Function {
            Ok(self.parse_function_declaration(block)?)
        } else if next.kind == TokenKind::Return {
            Ok(self.parse_return_statement(block)?)
        } else if next.kind == TokenKind::If {
            Ok(self.parse_if_statement(block)?)
        } else if next.kind == TokenKind::Identifier {
            let ident = self.parse_identifier(block)?;
            if self.parser.peek(0)?.kind == TokenKind::Assign {
                Ok(self.parse_assignment_statement(ident, block)?)
            } else if self.parser.peek(0)?.kind == TokenKind::LeftParen {
                Ok(self.parse_call(ident, block)?)
            } else {
                Err(SyntaxError::IdentifierAsStatement { span: next.span }.into())
            }
        } else if next.kind == TokenKind::Comment {
            let comment = self.parser.consume()?;
            Ok(self.ast.add(ASTNode::Comment {
                span: comment.span,
                value: comment.value.into(),
            }))
        } else {
            Err(SyntaxError::NonStatementToken {
                got: next.kind,
                span: next.span,
            }
            .into())
        }
    }

    fn parse_variable_declaration(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Let)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let primitive = self.parser.try_parse_primitive_type()?;
        self.parser.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = self.symbols.define_symbol(
            self.parser.strings,
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
        self.parser.expect(TokenKind::Const)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let const_type = self
            .parser
            .try_parse_primitive_type()?
            .ok_or(SyntaxError::MissingConstantType { span: token.span })?;
        self.parser.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = Symbol::Constant { const_type };
        let symbol_id = self.symbols.define_symbol(
            self.parser.strings,
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
                span: self.parser.peek(0).unwrap().span,
            }
            .into());
        }
        self.parser.expect(TokenKind::Function)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let symbol_id = self.symbols.define_symbol(
            self.parser.strings,
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
        self.parser.expect(TokenKind::LeftParen)?;
        // First argument
        if self.parser.peek(0)?.kind != TokenKind::RightParen {
            let arg = self.parse_function_argument(function_block)?;
            self.ast.append_child(function, arg);
        }
        // Rest of the arguments
        while self.parser.accept(TokenKind::Comma)?.is_some() {
            let arg = self.parse_function_argument(function_block)?;
            self.ast.append_child(function, arg);
        }
        self.parser.expect(TokenKind::RightParen)?;
        if let Some(primitive) = self.parser.try_parse_primitive_type()? {
            // Update return type
            if let ASTNode::FunctionDeclaration { symbol, .. } = self.ast.get_mut(function).unwrap()
            {
                if let Some(Symbol::Function { return_type, .. }) = self.symbols.get_mut(*symbol) {
                    *return_type = Some(primitive);
                }
            }
        }
        while self.parser.peek(0)?.kind != TokenKind::End {
            let stmt = self.parse_statement(function_block)?;
            self.ast.append_child(function, stmt);
        }
        self.parser.expect(TokenKind::End)?;
        Ok(function)
    }

    fn parse_function_argument(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let token = self.parser.expect(TokenKind::Identifier)?;
        let primitive = self.parser.try_parse_primitive_type()?;
        // Check duplicated argument
        if self
            .symbols
            .find_in_block(self.parser.strings, token.value.into(), block)
            .is_some()
        {
            return Err(SyntaxError::DuplicatedArgument { span: token.span }.into());
        }
        let symbol = self.symbols.define_symbol(
            self.parser.strings,
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
        self.parser.expect(TokenKind::Return)?;
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
        while ![TokenKind::End, TokenKind::Else, TokenKind::Elif]
            .contains(&self.parser.peek(0)?.kind)
        {
            let stmt = self.parse_statement(block)?;
            self.ast.append_child(body, stmt);
        }
        self.ast.append_child(if_node, body);
        Ok(())
    }

    fn parse_if_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::If)?;
        let node = self.ast.add(ASTNode::If);
        // If condition
        let condition = self.parse_expression(0, block)?;
        self.ast.append_child(node, condition);
        self.parser.expect(TokenKind::Then)?;
        // If body
        let if_block = self.symbols.add_block(Some(block));
        self.parse_and_append_if_body(node, if_block)?;
        while self.parser.accept(TokenKind::Elif)?.is_some() {
            // Elif condition
            let condition = self.parse_expression(0, block)?;
            self.ast.append_child(node, condition);
            self.parser.expect(TokenKind::Then)?;
            // Elif body
            self.parse_and_append_if_body(node, if_block)?;
        }
        if self.parser.accept(TokenKind::Else)?.is_some() {
            // Else body
            self.parse_and_append_if_body(node, block)?;
        }
        self.parser.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_for_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::For)?;
        let for_block = self.symbols.add_block(Some(block));
        let token = self.parser.expect(TokenKind::Identifier)?;
        self.parser.expect(TokenKind::In)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::For);
        // self.ast.append_child(node, )?;
        self.ast.append_child(node, expr);
        self.parser.expect(TokenKind::Do)?;
        while self.parser.peek(0)?.kind != TokenKind::End {
            let stmt = self.parse_statement(for_block)?;
            self.ast.append_child(node, stmt);
        }
        self.parser.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_assignment_statement(
        &mut self,
        ident: ASTNodeId,
        block: BlockId,
    ) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::Assignment);
        self.ast.append_child(node, ident);
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_call(&mut self, ident: ASTNodeId, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let node = self.ast.add(ASTNode::Call);
        self.ast.append_child(node, ident);
        self.parser.expect(TokenKind::LeftParen)?;
        if self.parser.peek(0)?.kind != TokenKind::RightParen {
            let expr = self.parse_expression(0, block)?;
            self.ast.append_child(node, expr);
        }
        while self.parser.accept(TokenKind::Comma)?.is_some() {
            let expr = self.parse_expression(0, block)?;
            self.ast.append_child(node, expr);
        }
        self.parser.expect(TokenKind::RightParen)?;
        Ok(node)
    }

    fn parse_import(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Import)?;
        let path = self.parser.expect(TokenKind::Literal)?;
        self.parser.expect(TokenKind::As)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let symbol = self.symbols.define_symbol(
            self.parser.strings,
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

        while self.parser.peek(0)?.kind == TokenKind::Import {
            let import = self.parse_import(global_block)?;
            self.ast.append_child(self.ast.root(), import);
        }

        while self.parser.peek(0)?.kind != TokenKind::EOF {
            let stmt = self.parse_statement(global_block)?;
            self.ast.append_child(self.ast.root(), stmt);
        }

        self.parser.expect(TokenKind::EOF)?;

        Ok(())
    }

    pub(crate) fn evaluate(
        ast: &mut AST,
        symbols: &mut SymbolTable,
        strings: &mut StringTable,
        lexer: &mut Lexer,
        source: &mut SourceStream,
    ) -> Result<(), CompileError> {
        let mut parser = ASTParser {
            parser: &mut Parser {
                lexer,
                strings,
                source,
            },
            ast,
            symbols,
        };
        parser.parse()?;
        Ok(())
    }
}

pub(crate) struct ImportExportParser<'a, S: Iterator<Item = (char, Location)>> {
    parser: &'a mut Parser<'a, S>,
    exports: Option<&'a mut ExportTable>,
    compilation_unit: &'a mut CompilationUnit,
    module: ModuleId,
}

impl<'a, S: Iterator<Item = (char, Location)>> ImportExportParser<'a, S> {
    fn parse(&mut self) -> Result<(), CompileError> {
        loop {
            // Import or exports can happen outside the global scope which is
            // illegal. However, the next MIR generation will failed in this
            // case. So we assume that we can continue the parsing.
            let token = self.parser.consume()?;
            if token.kind == TokenKind::EOF {
                break;
            } else if token.kind == TokenKind::Import {
            } else if let Some(exports) = self.exports {
                match token.kind {
                    TokenKind::From => {
                        let token = self.parser.expect(TokenKind::Literal)?;
                        // token.
                    }
                    TokenKind::Export => {
                        let token = self.parser.consume()?;
                        match token.kind {
                            TokenKind::Const => {
                                let token = self.parser.expect(TokenKind::Identifier)?;
                                let name = UID::new(self.parser.strings.get(token.value.into()));
                                let ty = self.parser.try_parse_primitive_type()?.ok_or(
                                    CompileError::Syntax(SyntaxError::MissingConstantType {
                                        span: token.span,
                                    }),
                                )?;
                                exports.add(self.module, name, Export::Constant { ty });
                            }
                            TokenKind::Function => {}
                            _ => {
                                return Err(CompileError::Syntax(
                                    SyntaxError::UnexpectedExportToken {
                                        span: token.span,
                                        got: token.kind,
                                    },
                                ))
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub(crate) fn evaluate(
        strings: &mut StringTable,
        lexer: &mut Lexer,
        source: &mut SourceStream,
        compilation_unit: &mut CompilationUnit,
        exports: Option<&mut ExportTable>,
        module: ModuleId,
    ) -> Result<(), CompileError> {
        let mut parser = ImportExportParser {
            parser: &mut Parser {
                lexer,
                strings,
                source,
            },
            compilation_unit,
            exports,
            module,
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
        ASTParser::<SourceStream>::evaluate(
            &mut ast,
            &mut symbols,
            &mut strings,
            &mut Lexer::new(false),
            &mut source,
        )
        .unwrap();
        ast.print();
    }

    #[test]
    fn test_if_body() {
        let mut strings = StringTable::default();
        let mut symbols = SymbolTable::default();
        let mut source = SourceStream::new(
            r#"
        let x = 2
        if x > 3 then
            print('hello')
        elif x == 2 then
            print('yes')
        else
            x = 2
        end
        "#,
        );
        let mut ast = AST::default();
        ASTParser::<SourceStream>::evaluate(
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
