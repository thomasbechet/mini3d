use super::{
    literal::Literal,
    operator::{BinaryOperator, UnaryOperator},
    string::StringId,
    symbol::{BlockId, SymbolId},
    token::Span,
};

#[derive(Debug)]
pub enum ASTNode {
    Program,
    Import {
        path: Span,
        symbol: SymbolId,
    },
    CompoundStatement, // STMT-0, STMT-1, STMT-2, ...
    Literal(Literal),
    Identifier {
        span: Span,
        symbol: SymbolId,
    },
    MemberLookup {
        span: Span,
    }, // PARENT-0
    ReturnStatement, // EXPR
    IfStatement,     // CONDITION-0, BODY-0, CONDITION-1, BODY-1, ...
    IfBody,
    ForStatement, // IDENTIFIER-0, GENERATOR-0, BODY-0
    ForBody,
    CommentStatement {
        span: Span,
        value: StringId,
    },
    FunctionDeclaration {
        span: Span,
        symbol: SymbolId,
        function_block: BlockId,
    }, // ARG-0, ARG-1, ..., COMPOUNT-STMT
    FunctionArgument {
        span: Span,
        symbol: SymbolId,
    },
    VariableDeclaration {
        span: Span,
        symbol: SymbolId,
    }, // EXPR
    ConstantDeclaration {
        span: Span,
        symbol: SymbolId,
    }, // EXPR (const)
    Call,
    Assignment,                     // IDENTIFIER-0, EXPR-0
    BinaryOperator(BinaryOperator), // LEFT-EXPR-0, RIGHT-EXPR-1
    UnaryOperator(UnaryOperator),   // EXPR
}

pub struct ASTEntry {
    pub(crate) node: ASTNode,
    pub(crate) parent: Option<ASTNodeId>,
    pub(crate) first_child: Option<ASTNodeId>,
    pub(crate) last_child: Option<ASTNodeId>,
    pub(crate) next_sibling: Option<ASTNodeId>,
}

pub(crate) type ASTNodeId = usize;

pub struct ASTChildIterator<'a> {
    next: Option<ASTNodeId>,
    ast: &'a AST,
}

impl<'a> Iterator for ASTChildIterator<'a> {
    type Item = (ASTNodeId, &'a ASTNode);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next {
            self.next = self.ast.entries[next].next_sibling;
            Some((next, &self.ast.entries[next].node))
        } else {
            None
        }
    }
}

pub struct AST {
    root: ASTNodeId,
    entries: Vec<ASTEntry>,
    strings: String,
}

impl AST {
    pub(crate) fn new() -> Self {
        Self {
            root: 0,
            entries: vec![ASTEntry {
                node: ASTNode::Program,
                parent: None,
                first_child: None,
                last_child: None,
                next_sibling: None,
            }],
            strings: String::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.root = 0;
        self.entries.clear();
        self.entries.push(ASTEntry {
            node: ASTNode::Program,
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
        });
        self.strings.clear();
    }

    pub(crate) fn root(&self) -> ASTNodeId {
        self.root
    }

    pub(crate) fn add(&mut self, node: ASTNode) -> ASTNodeId {
        let node = ASTEntry {
            node,
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
        };
        self.entries.push(node);
        self.entries.len() - 1
    }

    pub(crate) fn append_child(&mut self, parent: ASTNodeId, child: ASTNodeId) {
        if let Some(last_child) = self.entries[parent].last_child {
            self.entries[last_child].next_sibling = Some(child);
            self.entries[parent].last_child = Some(child);
        } else {
            self.entries[parent].first_child = Some(child);
            self.entries[parent].last_child = Some(child);
        }
        self.entries[child].parent = Some(parent);
    }

    pub(crate) fn iter_childs(&self, node: ASTNodeId) -> ASTChildIterator {
        let next = self.entries[node].first_child;
        ASTChildIterator { next, ast: self }
    }

    pub(crate) fn get_mut(&mut self, node: ASTNodeId) -> Option<&mut ASTNode> {
        self.entries.get_mut(node).map(|e| &mut e.node)
    }

    fn print_node(&self, node: ASTNodeId, indent: usize) {
        let entry = &self.entries[node];
        println!("{}- {:?}", " ".repeat(indent), entry.node);
        for (child, _) in self.iter_childs(node) {
            self.print_node(child, indent + 4);
        }
    }

    pub fn print(&self) {
        self.print_node(self.root(), 0);
    }
}
