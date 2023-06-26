use crate::{
    script::{
        constant::ConstantId,
        frontend::{
            error::{CompileError, SyntaxError},
            export::ExportId,
            mir::primitive::PrimitiveType,
        },
    },
    uid::UID,
};

use super::token::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceSymbolId(u32);

impl From<usize> for SourceSymbolId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl SourceSymbolId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SourceBlockId(u32);

impl SourceBlockId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub(crate) enum SourceSymbol {
    Function {
        return_type: Option<PrimitiveType>,
        first_arg: Option<SourceSymbolId>,
    },
    FunctionArgument {
        arg_type: PrimitiveType,
        next_arg: Option<SourceSymbolId>,
    },
    Variable {
        var_type: Option<PrimitiveType>,
    },
    Constant {
        value: Option<ConstantId>,
    },
    Import(ExportId),
    Module {
        path: ConstantId,
    },
}

#[derive(Debug)]
pub(crate) struct SourceBlockEntry {
    parent: Option<SourceBlockId>,
    last: Option<SourceSymbolId>,
    previous_scope_symbol: Option<SourceSymbolId>,
}

#[derive(Debug)]
pub(crate) struct SourceSymbolEntry {
    pub(crate) ident: ConstantId,
    pub(crate) hash: UID,
    pub(crate) block: SourceBlockId,
    pub(crate) symbol: Option<SourceSymbol>,
    pub(crate) previous_in_block: Option<SourceSymbolId>,
    pub(crate) previous_in_scope: Option<SourceSymbolId>,
}

#[derive(Default, Debug)]
pub(crate) struct SourceSymbolTable {
    pub(crate) symbols: Vec<SourceSymbolEntry>,
    pub(crate) blocks: Vec<SourceBlockEntry>,
    pub(crate) symbol_state: usize,
    pub(crate) block_state: usize,
}

impl SourceSymbolTable {
    pub(crate) const GLOBAL_BLOCK: SourceBlockId = SourceBlockId(0);

    pub(crate) fn clear(&mut self) {
        self.symbols.clear();
        self.blocks.clear();
        self.symbol_state = 0;
        self.block_state = 0;
    }

    pub(crate) fn push_state(&mut self) {}

    pub(crate) fn pop_state(&mut self) {}

    fn add_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        block: SourceBlockId,
        symbol: Option<SourceSymbol>,
    ) -> SourceSymbolId {
        // Prepare entry
        let value = strings.slice(ident);
        let mut entry = SourceSymbolEntry {
            symbol,
            hash: value.into(),
            ident,
            block,
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
        SourceSymbolId(self.symbols.len() as u32 - 1)
    }

    pub(crate) fn define_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        block: SourceBlockId,
        span: Span,
        symbol: SourceSymbol,
        allow_shadowing: bool,
    ) -> Result<SourceSymbolId, CompileError> {
        let exist = self.find_in_scope(strings, ident, block).is_some();
        if exist && !allow_shadowing {
            return Err(SyntaxError::SymbolAlreadyDefined { span }.into());
        }
        Ok(self.add_symbol(strings, ident, block, Some(symbol)))
    }

    pub(crate) fn get_mut(&mut self, id: SourceSymbolId) -> Option<&mut SourceSymbol> {
        self.symbols
            .get_mut(id.0 as usize)
            .and_then(|s| s.symbol.as_mut())
    }

    pub(crate) fn lookup_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        block: SourceBlockId,
    ) -> SourceSymbolId {
        if let Some(id) = self.find_in_scope(strings, ident, block) {
            id
        } else {
            self.add_symbol(strings, ident, block, None)
        }
    }

    pub(crate) fn add_block(&mut self, parent: Option<SourceBlockId>) -> SourceBlockId {
        // Compute previous_scope_symbol
        let previous_scope_symbol = parent.and_then(|p| {
            let parent = self.blocks.get(p.index()).unwrap();
            parent.last.or(parent.previous_scope_symbol)
        });

        // Add scope
        self.blocks.push(SourceBlockEntry {
            parent,
            last: None,
            previous_scope_symbol,
        });
        SourceBlockId(self.blocks.len() as u32 - 1)
    }

    pub(crate) fn find_in_block(
        &self,
        strings: &StringTable,
        ident: StringId,
        block: SourceBlockId,
    ) -> Option<SourceSymbolId> {
        let mut entry = self.blocks.get(block.index()).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = self.symbols.get(id.index()).unwrap();
                if symbol.hash == strings.slice(ident).into() {
                    return Some(id);
                }
                entry = self.symbols[id.index()].previous_in_block;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn find_in_scope(
        &self,
        strings: &StringTable,
        ident: StringId,
        block: SourceBlockId,
    ) -> Option<SourceSymbolId> {
        let mut entry = self.blocks.get(block.index()).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols.get(id.index()).unwrap();
                if symbol.hash == strings.slice(ident).into() {
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
            println!(
                "- [{}] '{}' {:?}",
                i,
                strings.slice(entry.ident),
                entry.symbol
            );
        }
        println!("BLOCKS:");
        for (i, block) in self.blocks.iter().enumerate() {
            println!("- {} {:?}", i, block);
        }
    }
}
