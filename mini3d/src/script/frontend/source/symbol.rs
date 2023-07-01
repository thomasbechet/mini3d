use crate::{
    script::{
        export::{Export, ExportId, ExportTable},
        frontend::{
            error::{CompileError, SyntaxError},
            source::token::TokenKind,
        },
        mir::primitive::PrimitiveType,
        module::ModuleId,
    },
    uid::UID,
};

use super::{strings::StringTable, token::Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

impl From<usize> for SymbolId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl SymbolId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BlockId(u32);

impl BlockId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub(crate) enum Symbol {
    Function {
        return_type: Option<PrimitiveType>,
        first_arg: Option<SymbolId>,
        exported: Option<ExportId>,
    },
    FunctionArgument {
        arg_type: PrimitiveType,
        next_arg: Option<SymbolId>,
    },
    Constant {
        const_type: PrimitiveType,
        exported: Option<ExportId>,
    },
    Variable {
        var_type: Option<PrimitiveType>,
    },
    Module {
        module: ModuleId,
    },
}

#[derive(Debug)]
pub(crate) struct BlockEntry {
    parent: Option<BlockId>,
    last: Option<SymbolId>,
    previous_scope_symbol: Option<SymbolId>,
}

#[derive(Debug)]
pub(crate) struct SymbolEntry {
    pub(crate) uid: UID,
    pub(crate) token: Token,
    pub(crate) block: BlockId,
    pub(crate) symbol: Option<Symbol>,
    pub(crate) previous_in_block: Option<SymbolId>,
    pub(crate) previous_in_scope: Option<SymbolId>,
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    pub(crate) symbols: Vec<SymbolEntry>,
    pub(crate) blocks: Vec<BlockEntry>,
    pub(crate) symbol_state: usize,
    pub(crate) block_state: usize,
}

impl SymbolTable {
    pub(crate) const GLOBAL_BLOCK: BlockId = BlockId(0);

    pub(crate) fn clear(&mut self) {
        self.symbols.clear();
        self.blocks.clear();
        self.symbol_state = 0;
        self.block_state = 0;
    }

    fn add_symbol(
        &mut self,
        uid: UID,
        token: Token,
        block: BlockId,
        symbol: Option<Symbol>,
    ) -> SymbolId {
        // Prepare entry
        let mut entry = SymbolEntry {
            uid,
            token,
            block,
            symbol,
            previous_in_block: None,
            previous_in_scope: None,
        };
        let id = self.symbols.len();

        // Update previous in scope
        if let Some(last) = self.blocks[block.index()].last {
            entry.previous_in_scope = Some(last);
        } else {
            entry.previous_in_scope = self.blocks[block.index()].previous_scope_symbol;
        }

        // Update block list
        let block_entry = self.blocks.get_mut(block.index()).unwrap();
        entry.previous_in_block = block_entry.last;
        block_entry.last = Some(id.into());

        // Add symbol
        self.symbols.push(entry);
        SymbolId(self.symbols.len() as u32 - 1)
    }

    pub(crate) fn define_symbol(
        &mut self,
        uid: UID,
        token: Token,
        block: BlockId,
        symbol: Symbol,
    ) -> Result<SymbolId, CompileError> {
        // Check if symbol already exist in scope
        let found = self.find_in_scope(uid, block);
        match symbol {
            Symbol::Function { .. } | Symbol::Constant { .. } | Symbol::Module { .. } => {
                assert!(block == Self::GLOBAL_BLOCK);
                if let Some(id) = found {
                    // Constant and function shadowing is not allowed, we must check definition.
                    if self.symbols[id.index()].symbol.is_some() {
                        // Already defined
                        return Err(SyntaxError::SymbolAlreadyDefined { span: token.span }.into());
                    } else {
                        // Update symbol
                        self.symbols[id.index()].symbol = Some(symbol);
                        return Ok(id);
                    }
                } else {
                    // Not defined or declared
                    return Ok(self.add_symbol(uid, token, block, Some(symbol)));
                }
            }
            Symbol::Variable { .. } | Symbol::FunctionArgument { .. } => {
                return Ok(self.add_symbol(uid, token, block, Some(symbol)));
            }
        }
    }

    pub(crate) fn import_symbol(
        &mut self,
        block: BlockId,
        token: Token,
        id: ExportId,
        exports: &ExportTable,
    ) -> Result<(), CompileError> {
        match exports.get(id).unwrap() {
            Export::Constant { name, ty } => {
                let symbol = Symbol::Constant {
                    const_type: *ty,
                    exported: Some(id),
                };
                self.define_symbol(*name, token, block, symbol)?;
            }
            Export::Function {
                name,
                ty,
                first_arg,
            } => {
                // Define function arguments
                let mut next = *first_arg;
                let mut previous: Option<SymbolId> = None;
                let mut first_arg = None;
                while let Some(arg) = next {
                    match exports.get(arg).unwrap() {
                        Export::FunctionArgument { name, ty, next_arg } => {
                            // Add symbol
                            let id = self.define_symbol(
                                *name,
                                token,
                                block,
                                Symbol::FunctionArgument {
                                    arg_type: *ty,
                                    next_arg: None,
                                },
                            )?;
                            // Update previous argument link
                            if let Some(previous) = previous {
                                match self.symbols[previous.0 as usize].symbol.as_mut().unwrap() {
                                    Symbol::FunctionArgument { next_arg, .. } => {
                                        *next_arg = Some(id);
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            // Keep reference to first argument symbol
                            if first_arg.is_none() {
                                first_arg = Some(id);
                            }
                            previous = Some(id);
                            next = *next_arg;
                        }
                        _ => unreachable!(),
                    }
                }
                // Define function
                self.define_symbol(
                    *name,
                    token,
                    block,
                    Symbol::Function {
                        return_type: *ty,
                        first_arg,
                        exported: Some(id),
                    },
                )?;
            }
            _ => { /* Ignore function arguments */ }
        }
        Ok(())
    }

    pub(crate) fn import_all_symbols(
        &mut self,
        block: BlockId,
        token: Token,
        module: ModuleId,
        exports: &ExportTable,
    ) -> Result<(), CompileError> {
        for id in exports.iter(module) {
            self.import_symbol(block, token, id, exports)?;
        }
        Ok(())
    }

    pub(crate) fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols
            .get_mut(id.0 as usize)
            .and_then(|s| s.symbol.as_mut())
    }

    pub(crate) fn lookup_symbol(&mut self, uid: UID, token: Token, block: BlockId) -> SymbolId {
        if let Some(id) = self.find_in_scope(uid, block) {
            id
        } else {
            self.add_symbol(uid, token, Self::GLOBAL_BLOCK, None)
        }
    }

    pub(crate) fn add_block(&mut self, parent: Option<BlockId>) -> BlockId {
        // Compute previous_scope_symbol
        let previous_scope_symbol = parent.and_then(|p| {
            let parent = self.blocks.get(p.index()).unwrap();
            parent.last.or(parent.previous_scope_symbol)
        });

        // Add scope
        self.blocks.push(BlockEntry {
            parent,
            last: None,
            previous_scope_symbol,
        });
        BlockId(self.blocks.len() as u32 - 1)
    }

    pub(crate) fn find_in_block(&self, uid: UID, block: BlockId) -> Option<SymbolId> {
        let mut entry = self.blocks.get(block.index()).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = self.symbols.get(id.index()).unwrap();
                if symbol.uid == uid {
                    return Some(id);
                }
                entry = self.symbols[id.index()].previous_in_block;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn find_in_scope(&self, uid: UID, block: BlockId) -> Option<SymbolId> {
        let block = self.blocks.get(block.index()).unwrap();
        let mut entry = block.last.or(block.previous_scope_symbol);
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols.get(id.index()).unwrap();
                if symbol.uid == uid {
                    return Some(id);
                }
                entry = self.symbols[id.index()].previous_in_scope;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn print(&self, strings: &StringTable) {
        println!("SYMBOLS:");
        for (i, entry) in self.symbols.iter().enumerate() {
            let ident = if entry.token.kind == TokenKind::Multiply {
                "*"
            } else {
                strings.get(entry.token.value.into())
            };
            println!(
                "- [{}] '{}' {:?} UID: {}",
                i, ident, entry.symbol, entry.uid
            );
        }
        println!("BLOCKS:");
        for (i, block) in self.blocks.iter().enumerate() {
            println!("- [{}] {:?}", i, block);
        }
    }
}
