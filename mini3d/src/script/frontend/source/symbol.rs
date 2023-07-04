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
        return_type: PrimitiveType,
        first_arg: Option<SymbolId>,
        exported: bool,
    },
    FunctionArgument {
        arg_type: PrimitiveType,
        next_arg: Option<SymbolId>,
    },
    Constant {
        const_type: PrimitiveType,
        exported: bool,
    },
    Variable {
        var_type: Option<PrimitiveType>,
    },
    Module {
        module: ModuleId,
    },
    External {
        export: ExportId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockKind {
    Global,
    Function,
    While,
    For,
    If,
    Loop,
}

#[derive(Debug)]
pub(crate) struct BlockEntry {
    kind: BlockKind,
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
            Symbol::Function { .. }
            | Symbol::Constant { .. }
            | Symbol::Module { .. }
            | Symbol::External { .. } => {
                assert!(block == Self::GLOBAL_BLOCK);
                if let Some(id) = found {
                    // Constant and function shadowing is not allowed, we must check definition.
                    if self.symbols[id.index()].symbol.is_some() {
                        // Already defined
                        Err(SyntaxError::SymbolAlreadyDefined { span: token.span }.into())
                    } else {
                        // Update symbol
                        self.symbols[id.index()].symbol = Some(symbol);
                        Ok(id)
                    }
                } else {
                    // Not defined or declared
                    Ok(self.add_symbol(uid, token, block, Some(symbol)))
                }
            }
            Symbol::Variable { .. } | Symbol::FunctionArgument { .. } => {
                Ok(self.add_symbol(uid, token, block, Some(symbol)))
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
            Export::Constant { name, .. } => {
                self.define_symbol(*name, token, block, Symbol::External { export: id })?;
            }
            Export::Function { name, .. } => {
                self.define_symbol(*name, token, block, Symbol::External { export: id })?;
            }
            Export::Argument { .. } => {}
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

    pub(crate) fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols
            .get(id.0 as usize)
            .and_then(|s| s.symbol.as_ref())
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

    pub(crate) fn add_block(&mut self, kind: BlockKind, parent: Option<BlockId>) -> BlockId {
        // Compute previous_scope_symbol
        let previous_scope_symbol = parent.and_then(|p| {
            let parent = self.blocks.get(p.index()).unwrap();
            parent.last.or(parent.previous_scope_symbol)
        });

        // Add scope
        self.blocks.push(BlockEntry {
            kind,
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

    fn check_in(&self, entry: BlockId, kinds: &[BlockKind]) -> bool {
        let mut block = Some(entry);
        while let Some(current) = block {
            let entry = &self.blocks[current.index()];
            if kinds.contains(&entry.kind) {
                return true;
            }
            block = entry.parent;
        }
        false
    }

    pub(crate) fn check_in_loop(&self, entry: BlockId) -> bool {
        self.check_in(entry, &[BlockKind::Loop, BlockKind::While, BlockKind::For])
    }

    pub(crate) fn check_in_function(&self, entry: BlockId) -> bool {
        self.check_in(entry, &[BlockKind::Function])
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
                "- [{}] '{}' {:?} UID: {} BLOCK: {:?}",
                i, ident, entry.symbol, entry.uid, entry.block
            );
        }
        println!("BLOCKS:");
        for (i, block) in self.blocks.iter().enumerate() {
            println!("- [{}] {:?}", i, block);
        }
    }
}
