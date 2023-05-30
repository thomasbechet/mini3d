use crate::uid::UID;

#[derive(Debug, PartialEq)]
pub(crate) enum PrimitiveType {
    Boolean,
    Integer,
    Float,
    String,
    Entity,
    Object,
}

impl PrimitiveType {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "bool" => Some(Self::Boolean),
            "int" => Some(Self::Integer),
            "float" => Some(Self::Float),
            "string" => Some(Self::String),
            "entity" => Some(Self::Entity),
            "object" => Some(Self::Object),
            _ => None,
        }
    }
}

pub(crate) type SymbolId = u16;
pub(crate) type BlockId = u16;

#[derive(Debug, PartialEq)]
pub(crate) enum SymbolKind {
    Function { return_type: Option<PrimitiveType> },
    Variable { var_type: Option<PrimitiveType> },
    Constant { const_type: Option<PrimitiveType> },
    Import,
    Undefined,
}

#[derive(Debug)]
pub(crate) struct BlockEntry {
    parent: Option<BlockId>,
    last: Option<SymbolId>,
    previous_scope_symbol: Option<SymbolId>,
}

#[derive(Debug)]
pub(crate) struct Symbol {
    pub(crate) ident: UID,
    pub(crate) kind: SymbolKind,
    pub(crate) block: BlockId,
}

#[derive(Debug)]
pub(crate) struct SymbolEntry {
    pub(crate) symbol: Symbol,
    pub(crate) previous_in_block: Option<SymbolId>,
    pub(crate) previous_in_scope: Option<SymbolId>,
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    pub(crate) symbols: Vec<SymbolEntry>,
    pub(crate) blocks: Vec<BlockEntry>,
}

impl SymbolTable {
    pub(crate) const GLOBAL_BLOCK: BlockId = 0;

    pub(crate) fn declare_symbol(
        &mut self,
        ident: &str,
        kind: SymbolKind,
        block: BlockId,
    ) -> SymbolId {
        // Prepare entry
        let mut entry = SymbolEntry {
            symbol: Symbol {
                ident: ident.into(),
                kind,
                block,
            },
            previous_in_block: None,
            previous_in_scope: None,
        };
        let id = self.symbols.len() as u16;

        // Update previous in scope
        if let Some(last) = self.blocks[block as usize].last {
            entry.previous_in_scope = Some(last);
        } else {
            entry.previous_in_scope = self.blocks[block as usize].previous_scope_symbol;
        }

        // Update block list
        let block_entry = self.blocks.get_mut(block as usize).unwrap();
        entry.previous_in_block = block_entry.last;
        block_entry.last = Some(id);

        // Add symbol
        self.symbols.push(entry);
        self.symbols.len() as u16 - 1
    }

    pub(crate) fn define_block(&mut self, parent: Option<BlockId>) -> BlockId {
        // Compute previous_scope_symbol
        let previous_scope_symbol = parent.and_then(|p| {
            let parent = self.blocks.get(p as usize).unwrap();
            parent.last.or(parent.previous_scope_symbol)
        });

        // Add scope
        self.blocks.push(BlockEntry {
            parent,
            last: None,
            previous_scope_symbol,
        });
        self.blocks.len() as u16 - 1
    }

    pub(crate) fn find_in_block(&self, ident: &str, block: BlockId) -> Option<SymbolId> {
        let mut entry = self.blocks.get(block as usize).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols[id as usize].symbol;
                if symbol.ident == ident.into() {
                    return Some(id);
                }
                entry = self.symbols[id as usize].previous_in_block;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn lookup(&mut self, ident: &str, block: BlockId) -> SymbolId {
        let mut entry = self.blocks.get(block as usize).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols[id as usize].symbol;
                if symbol.ident == ident.into() {
                    return id;
                }
                entry = self.symbols[id as usize].previous_in_scope;
            } else {
                // Symbol not found declare as undefined
                return self.declare_symbol(ident, SymbolKind::Undefined, block);
            }
        }
    }

    pub(crate) fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols.get(id as usize).unwrap().symbol
    }

    pub(crate) fn get_mut(&mut self, id: SymbolId) -> &mut Symbol {
        &mut self.symbols.get_mut(id as usize).unwrap().symbol
    }

    pub(crate) fn get_ident(&self, id: SymbolId) -> UID {
        self.symbols.get(id as usize).unwrap().symbol.ident
    }

    pub(crate) fn print(&self, source: &str) {
        println!("SYMBOLS:");
        for (i, entry) in self.symbols.iter().enumerate() {
            println!("- [{}] {:?}", i, entry.symbol);
        }
        println!("BLOCKS:");
        for (i, block) in self.blocks.iter().enumerate() {
            println!("- {} {:?}", i, block);
        }
    }
}
