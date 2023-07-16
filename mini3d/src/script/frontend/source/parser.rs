use crate::{
    script::{
        compiler::CompilationUnit,
        frontend::error::{CompileError, SemanticError, SyntaxError},
        mir::primitive::PrimitiveType,
        module::{ModuleId, ModuleSymbol, ModuleTable},
    },
    utils::uid::UID,
};

use super::{
    ast::{ASTNode, ASTNodeId, AST},
    lexer::Lexer,
    literal::Literal,
    operator::BinaryOperator,
    stream::SourceStream,
    strings::StringTable,
    symbol::{BlockId, BlockKind, Symbol, SymbolTable},
    token::{Location, Token, TokenKind, TokenValue},
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

    fn parse_identifier_list(
        &mut self,
        separator: TokenKind,
        mut f: impl FnMut((&str, Token)) -> Result<(), CompileError>,
    ) -> Result<usize, CompileError> {
        let mut count = 0;
        if let Some(token) = self.accept(TokenKind::Identifier)? {
            let name = self.strings.get(token.value.into());
            f((name, token))?;
            count += 1;
            while self.accept(separator)?.is_some() {
                let token = self.expect(TokenKind::Identifier)?;
                let name = self.strings.get(token.value.into());
                f((name, token))?;
                count += 1;
            }
        }
        Ok(count)
    }

    fn parse_function_argument_list(
        &mut self,
        separator: TokenKind,
        mut f: impl FnMut((&str, PrimitiveType, Token)) -> Result<(), CompileError>,
    ) -> Result<usize, CompileError> {
        let mut count = 0;
        if let Some(token) = self.accept(TokenKind::Identifier)? {
            let ty = self
                .try_parse_primitive_type()?
                .ok_or(CompileError::Syntax(SyntaxError::MissingArgumentType {
                    span: token.span,
                }))?;
            let name = match token.value {
                TokenValue::Identifier(ident) => self.strings.get(ident),
                _ => unreachable!(),
            };
            f((name, ty, token))?;
            count += 1;
            while self.accept(separator)?.is_some() {
                let token = self.expect(TokenKind::Identifier)?;
                let ty = self
                    .try_parse_primitive_type()?
                    .ok_or(CompileError::Syntax(SyntaxError::MissingArgumentType {
                        span: token.span,
                    }))?;
                let name = match token.value {
                    TokenValue::Identifier(ident) => self.strings.get(ident),
                    _ => unreachable!(),
                };
                f((name, ty, token))?;
                count += 1;
            }
        }
        Ok(count)
    }
}

pub(crate) struct ASTParser<'a, S: Iterator<Item = (char, Location)>> {
    parser: &'a mut Parser<'a, S>,
    ast: &'a mut AST,
    symbols: &'a mut SymbolTable,
    modules: &'a ModuleTable,
    module: ModuleId,
}

impl<'a, S: Iterator<Item = (char, Location)>> ASTParser<'a, S> {
    fn parse_member_lookup_chain(&mut self, child: ASTNodeId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Dot)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let node = self.ast.add(ASTNode::MemberLookup {
            span: token.span,
            ident: token.value.into(),
        });
        self.ast.append_child(node, child);
        if self.parser.peek(0)?.kind == TokenKind::Dot {
            self.parse_member_lookup_chain(node)
        } else {
            Ok(node)
        }
    }

    fn try_parse_member_lookup_chain(
        &mut self,
        node: ASTNodeId,
    ) -> Result<ASTNodeId, CompileError> {
        if self.parser.peek(0)?.kind == TokenKind::Dot {
            self.parse_member_lookup_chain(node)
        } else {
            Ok(node)
        }
    }

    fn try_parse_call(
        &mut self,
        node: ASTNodeId,
        block: BlockId,
    ) -> Result<ASTNodeId, CompileError> {
        if self.parser.peek(0)?.kind == TokenKind::LeftParen {
            self.parse_call(node, block)
        } else {
            Ok(node)
        }
    }

    fn parse_identifier(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let token = self.parser.expect(TokenKind::Identifier)?;
        let ident = self.parser.strings.get(token.value.into());
        let symbol = self.symbols.lookup_symbol(ident.into(), token, block);
        Ok(self.ast.add(ASTNode::Identifier {
            span: token.span,
            symbol,
        }))
    }

    fn parse_atom(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        let next = self.parser.peek(0)?;
        let node = match next.kind {
            TokenKind::Identifier => {
                let node = self.parse_identifier(block)?;
                let node = self.try_parse_member_lookup_chain(node)?;
                self.try_parse_call(node, block)?
            }
            TokenKind::PrimitiveType => {
                let token = self.parser.expect(TokenKind::PrimitiveType)?;
                let node = self.ast.add(ASTNode::PrimitiveType {
                    span: token.span,
                    ty: token.value.into(),
                });
                let node = self.try_parse_member_lookup_chain(node)?;
                self.try_parse_call(node, block)?
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
                Ok(self.parse_function_declaration(block, true)?)
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
            Ok(self.parse_function_declaration(block, false)?)
        } else if next.kind == TokenKind::Return {
            if !self.symbols.check_in_function(block) {
                return Err(SyntaxError::ReturnOutsideFunction { span: next.span }.into());
            }
            Ok(self.parse_return_statement(block)?)
        } else if next.kind == TokenKind::If {
            Ok(self.parse_if_statement(block)?)
        } else if next.kind == TokenKind::For {
            Ok(self.parse_for_statement(block)?)
        } else if next.kind == TokenKind::While {
            Ok(self.parse_while_statement(block)?)
        } else if next.kind == TokenKind::Loop {
            Ok(self.parse_loop_statement(block)?)
        } else if next.kind == TokenKind::Break {
            self.parser.consume()?;
            if !self.symbols.check_in_loop(block) {
                return Err(SyntaxError::BreakOutsideLoop { span: next.span }.into());
            }
            Ok(self.ast.add(ASTNode::Break))
        } else if next.kind == TokenKind::Continue {
            self.parser.consume()?;
            if !self.symbols.check_in_loop(block) {
                return Err(SyntaxError::ContinueOutsideLoop { span: next.span }.into());
            }
            Ok(self.ast.add(ASTNode::Continue))
        } else if next.kind == TokenKind::Identifier {
            let ident = self.parse_identifier(block)?;
            let ident = self.try_parse_member_lookup_chain(ident)?;
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
            self.parser.strings.get(token.value.into()).into(),
            token,
            block,
            Symbol::Variable {
                var_type: primitive,
            },
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
        exported: bool,
    ) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Const)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let ident = UID::new(self.parser.strings.get(token.value.into()));
        let const_type = self
            .parser
            .try_parse_primitive_type()?
            .ok_or(SyntaxError::MissingConstantType { span: token.span })?;
        self.parser.expect(TokenKind::Assign)?;
        let expr = self.parse_expression(0, block)?;
        let symbol = Symbol::Constant {
            const_type,
            exported,
        };
        let symbol_id = self.symbols.define_symbol(ident, token, block, symbol)?;
        if exported && block != SymbolTable::GLOBAL_BLOCK {
            return Err(SyntaxError::ExportOutsideOfGlobalScope { span: token.span }.into());
        }
        let node = self.ast.add(ASTNode::ConstantDeclaration {
            span: token.span,
            symbol: symbol_id,
        });
        self.ast.append_child(node, expr);
        Ok(node)
    }

    fn parse_function_declaration(
        &mut self,
        block: BlockId,
        exported: bool,
    ) -> Result<ASTNodeId, CompileError> {
        if block != SymbolTable::GLOBAL_BLOCK {
            return Err(SyntaxError::FunctionDeclarationOutsideOfGlobalScope {
                span: self.parser.peek(0).unwrap().span,
            }
            .into());
        }
        self.parser.expect(TokenKind::Function)?;
        let token = self.parser.expect(TokenKind::Identifier)?;
        let ident = self.parser.strings.get(token.value.into());
        let function_symbol = self.symbols.define_symbol(
            ident.into(),
            token,
            block,
            Symbol::Function {
                return_type: PrimitiveType::Nil,
                first_arg: None,
                exported,
            },
        )?;
        let function_block = self.symbols.add_block(BlockKind::Function, Some(block));
        let function = self.ast.add(ASTNode::FunctionDeclaration {
            span: token.span,
            symbol: function_symbol,
            function_block,
        });
        self.parser.expect(TokenKind::LeftParen)?;
        // Parse arguments
        let mut previous_argument = None;
        self.parser
            .parse_function_argument_list(TokenKind::Comma, |(name, ty, token)| {
                if self
                    .symbols
                    .find_in_block(name.into(), function_block)
                    .is_some()
                {
                    return Err(SyntaxError::DuplicatedArgument { span: token.span }.into());
                }
                let symbol = self.symbols.define_symbol(
                    name.into(),
                    token,
                    function_block,
                    Symbol::FunctionArgument {
                        arg_type: ty,
                        next_arg: None,
                    },
                )?;
                if let Some(previous_argument) = previous_argument {
                    match self.symbols.get_mut(previous_argument).unwrap() {
                        Symbol::FunctionArgument { next_arg, .. } => {
                            *next_arg = Some(symbol);
                        }
                        _ => unreachable!(),
                    }
                } else {
                    match self.symbols.get_mut(function_symbol).unwrap() {
                        Symbol::Function { first_arg, .. } => {
                            *first_arg = Some(symbol);
                        }
                        _ => unreachable!(),
                    }
                }
                previous_argument = Some(symbol);
                Ok(())
            })?;
        self.parser.expect(TokenKind::RightParen)?;
        if let Some(primitive) = self.parser.try_parse_primitive_type()? {
            // Update return type
            if let ASTNode::FunctionDeclaration { symbol, .. } = self.ast.get_mut(function).unwrap()
            {
                if let Some(Symbol::Function { return_type, .. }) = self.symbols.get_mut(*symbol) {
                    *return_type = primitive;
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
        let if_block = self.symbols.add_block(BlockKind::If, Some(block));
        let body = self.ast.add(ASTNode::IfBody { block: if_block });
        while ![TokenKind::End, TokenKind::Else, TokenKind::Elif]
            .contains(&self.parser.peek(0)?.kind)
        {
            let stmt = self.parse_statement(if_block)?;
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
        self.parse_and_append_if_body(node, block)?;
        while self.parser.accept(TokenKind::Elif)?.is_some() {
            // Elif condition
            let condition = self.parse_expression(0, block)?;
            self.ast.append_child(node, condition);
            self.parser.expect(TokenKind::Then)?;
            // Elif body
            self.parse_and_append_if_body(node, block)?;
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
        let for_block = self.symbols.add_block(BlockKind::For, Some(block));
        let token = self.parser.expect(TokenKind::Identifier)?;
        self.parser.expect(TokenKind::In)?;
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::For { block: for_block });
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

    fn parse_while_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::While)?;
        let while_block = self.symbols.add_block(BlockKind::While, Some(block));
        let expr = self.parse_expression(0, block)?;
        let node = self.ast.add(ASTNode::While { block: while_block });
        self.ast.append_child(node, expr);
        self.parser.expect(TokenKind::Do)?;
        while self.parser.peek(0)?.kind != TokenKind::End {
            let stmt = self.parse_statement(while_block)?;
            self.ast.append_child(node, stmt);
        }
        self.parser.expect(TokenKind::End)?;
        Ok(node)
    }

    fn parse_loop_statement(&mut self, block: BlockId) -> Result<ASTNodeId, CompileError> {
        self.parser.expect(TokenKind::Loop)?;
        let loop_block = self.symbols.add_block(BlockKind::Loop, Some(block));
        let node = self.ast.add(ASTNode::Loop { block: loop_block });
        while self.parser.peek(0)?.kind != TokenKind::End {
            let stmt = self.parse_statement(loop_block)?;
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

    fn parse_import(&mut self, block: BlockId) -> Result<(), CompileError> {
        self.parser.expect(TokenKind::Import)?;
        let path_token = self.parser.expect(TokenKind::Literal)?;
        match path_token.value {
            TokenValue::Literal(Literal::String(path)) => {
                let path = self.parser.strings.get(path);
                let module = self
                    .modules
                    .find(path.into())
                    .ok_or(CompileError::Semantic(SemanticError::ModuleNotFound {
                        span: path_token.span,
                    }))?;
                if module == self.module {
                    return Err(SemanticError::ImportSelf {
                        span: path_token.span,
                    }
                    .into());
                }
                self.parser.expect(TokenKind::As)?;
                let path_token = self.parser.expect(TokenKind::Identifier)?;
                let ident = self.parser.strings.get(path_token.value.into());
                self.symbols.define_symbol(
                    ident.into(),
                    path_token,
                    block,
                    Symbol::Module { module },
                )?;
            }
            _ => {
                return Err(SemanticError::ModuleNotFound {
                    span: path_token.span,
                }
                .into())
            }
        }
        Ok(())
    }

    fn parse_from(&mut self, block: BlockId) -> Result<(), CompileError> {
        self.parser.expect(TokenKind::From)?;
        let path_token = self.parser.expect(TokenKind::Literal)?;
        match path_token.value {
            TokenValue::Literal(Literal::String(path)) => {
                let path = self.parser.strings.get(path);
                let module = self
                    .modules
                    .find(path.into())
                    .ok_or(CompileError::Semantic(SemanticError::ModuleNotFound {
                        span: path_token.span,
                    }))?;
                self.parser.expect(TokenKind::Import)?;
                if let Some(multiply) = self.parser.accept(TokenKind::Multiply)? {
                    // Import all symbols
                    self.symbols
                        .import_all_symbols(block, multiply, module, self.modules)?;
                } else {
                    // Import selected symbols
                    let count =
                        self.parser
                            .parse_identifier_list(TokenKind::Comma, |(name, token)| {
                                let id = self.modules.find_symbol(module, name.into()).ok_or(
                                    CompileError::Semantic(SemanticError::ModuleImportNotFound {
                                        module,
                                        span: token.span,
                                    }),
                                )?;
                                self.symbols.import_symbol(block, token, id, self.modules)?;
                                Ok(())
                            })?;
                    if count == 0 {
                        return Err(SemanticError::MissingImportSymbols {
                            span: path_token.span,
                        }
                        .into());
                    }
                }
            }
            _ => {
                return Err(SemanticError::ModuleNotFound {
                    span: path_token.span,
                }
                .into())
            }
        }
        Ok(())
    }

    fn parse(&mut self) -> Result<(), CompileError> {
        let global_block = self.symbols.add_block(BlockKind::Global, None);

        while [TokenKind::Import, TokenKind::From].contains(&self.parser.peek(0)?.kind) {
            match self.parser.peek(0)?.kind {
                TokenKind::Import => self.parse_import(global_block)?,
                TokenKind::From => self.parse_from(global_block)?,
                _ => unreachable!(),
            }
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
        modules: &ModuleTable,
        module: ModuleId,
    ) -> Result<(), CompileError> {
        let mut parser = ASTParser {
            parser: &mut Parser {
                lexer,
                strings,
                source,
            },
            ast,
            symbols,
            modules,
            module,
        };
        parser.parse()?;
        Ok(())
    }
}

pub(crate) struct ImportExportParser<'a, S: Iterator<Item = (char, Location)>> {
    parser: &'a mut Parser<'a, S>,
    compilation_unit: &'a mut CompilationUnit,
    modules: &'a mut ModuleTable,
    module: ModuleId,
    depth: usize,
}

impl<'a, S: Iterator<Item = (char, Location)>> ImportExportParser<'a, S> {
    fn parse(&mut self) -> Result<(), CompileError> {
        loop {
            // Import or exports allowed only in the global scope. If an import/export occur inside a block, it will be ignored.
            // This will produce a failure during AST generation.
            let token = self.parser.consume()?;
            if token.kind == TokenKind::EOF {
                break;
            }

            if [TokenKind::Function, TokenKind::If, TokenKind::Do].contains(&token.kind) {
                // Detect entering in a block
                self.depth += 1;
            } else if self.depth > 0 {
                // Check inside a block
                if token.kind == TokenKind::End {
                    self.depth -= 1;
                }
            } else if [TokenKind::Import, TokenKind::From].contains(&token.kind) {
                match self.parser.expect(TokenKind::Literal)?.value {
                    TokenValue::Literal(Literal::String(path)) => {
                        let path = self.parser.strings.get(path);
                        let module =
                            self.modules
                                .find(path.into())
                                .ok_or(CompileError::Semantic(SemanticError::ModuleNotFound {
                                    span: token.span,
                                }))?;
                        if module == self.module {
                            return Err(SemanticError::ImportSelf { span: token.span }.into());
                        }
                        self.compilation_unit.add(module);
                    }
                    _ => return Err(SemanticError::ModuleNotFound { span: token.span }.into()),
                }
                if token.kind == TokenKind::From {
                    self.parser.expect(TokenKind::Import)?;
                }
            } else if token.kind == TokenKind::Export {
                let token = self.parser.consume()?;
                match token.kind {
                    TokenKind::Const => {
                        let token = self.parser.expect(TokenKind::Identifier)?;
                        let name = UID::new(self.parser.strings.get(token.value.into()));
                        let ty =
                            self.parser
                                .try_parse_primitive_type()?
                                .ok_or(CompileError::Syntax(SyntaxError::MissingConstantType {
                                    span: token.span,
                                }))?;
                        self.modules
                            .add_symbol(self.module, ModuleSymbol::Constant { ident: name, ty });
                    }
                    TokenKind::Function => {
                        let token = self.parser.expect(TokenKind::Identifier)?;
                        let name = UID::new(self.parser.strings.get(token.value.into()));
                        self.parser.expect(TokenKind::LeftParen)?;
                        let function_symbol = self.modules.add_symbol(
                            self.module,
                            ModuleSymbol::Function {
                                ident: name,
                                ty: PrimitiveType::Nil,
                                first_arg: None,
                            },
                        );
                        let mut previous_arg = None;
                        self.parser.parse_function_argument_list(
                            TokenKind::Comma,
                            |(name, ty, _)| {
                                let arg = self.modules.add_symbol(
                                    self.module,
                                    ModuleSymbol::Argument {
                                        ident: name.into(),
                                        ty,
                                        next_arg: None,
                                    },
                                );
                                if let Some(previous_arg) = previous_arg {
                                    match self.modules.get_symbol_mut(previous_arg).unwrap() {
                                        ModuleSymbol::Argument { next_arg, .. } => {
                                            *next_arg = Some(arg)
                                        }
                                        _ => unreachable!(),
                                    }
                                } else {
                                    match self.modules.get_symbol_mut(function_symbol).unwrap() {
                                        ModuleSymbol::Function { first_arg, .. } => {
                                            *first_arg = Some(arg)
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                previous_arg = Some(arg);
                                Ok(())
                            },
                        )?;
                        self.parser.expect(TokenKind::RightParen)?;
                        let function_ty = self
                            .parser
                            .try_parse_primitive_type()?
                            .unwrap_or(PrimitiveType::Nil);
                        match self.modules.get_symbol_mut(function_symbol).unwrap() {
                            ModuleSymbol::Function { ty, .. } => *ty = function_ty,
                            _ => unreachable!(),
                        }
                    }
                    _ => {
                        return Err(SyntaxError::UnexpectedExportToken {
                            span: token.span,
                            got: token.kind,
                        }
                        .into())
                    }
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
        modules: &mut ModuleTable,
        module: ModuleId,
    ) -> Result<(), CompileError> {
        let mut parser = ImportExportParser {
            parser: &mut Parser {
                lexer,
                strings,
                source,
            },
            compilation_unit,
            modules,
            module,
            depth: 0,
        };
        parser.parse()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::script::module::Module;

    use super::*;

    #[test]
    fn test_basic() {
        let mut strings = StringTable::default();
        let mut symbols = SymbolTable::default();
        let mut modules = ModuleTable::default();
        let module = modules.add(UID::null(), Module::Source { asset: UID::null() });
        let mut stream = SourceStream::new(
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
            &mut stream,
            &modules,
            module,
        )
        .unwrap();
        ast.print();
    }

    #[test]
    fn test_if_body() {
        let mut strings = StringTable::default();
        let mut symbols = SymbolTable::default();
        let mut modules = ModuleTable::default();
        let module = modules.add(UID::null(), Module::Source { asset: UID::null() });
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
        let mut source = SourceStream::new(
            r#"
        function test(a: int, b: int) end
        test(1)
        "#,
        );
        let mut ast = AST::default();
        ASTParser::<SourceStream>::evaluate(
            &mut ast,
            &mut symbols,
            &mut strings,
            &mut Lexer::new(false),
            &mut source,
            &modules,
            module,
        )
        .unwrap();
        ast.print();
        symbols.print(&strings);
    }
}
