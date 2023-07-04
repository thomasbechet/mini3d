use crate::script::mir::primitive::PrimitiveType;

use super::{
    literal::Literal,
    operator::{BinaryOperator, UnaryOperator},
    strings::StringId,
    symbol::{BlockId, SymbolId},
    token::Span,
};

#[derive(Debug)]
pub(crate) enum ASTNode {
    Program,
    Compound, // STMT-0, STMT-1, STMT-2, ...
    Literal(Literal),
    Identifier {
        span: Span,
        symbol: SymbolId,
    },
    PrimitiveType {
        span: Span,
        ty: PrimitiveType,
    },
    MemberLookup {
        span: Span,
        ident: StringId,
    }, // PARENT-0
    Break,
    Continue,
    Return, // EXPR
    If,     // CONDITION-0, BODY-0, CONDITION-1, BODY-1, ...
    IfBody {
        block: BlockId,
    },
    For {
        // IDENTIFIER-0, GENERATOR-0, BODY-0
        block: BlockId,
    },
    While {
        block: BlockId,
    },
    Loop {
        block: BlockId,
    },
    Comment {
        span: Span,
        value: StringId,
    },
    FunctionDeclaration {
        span: Span,
        symbol: SymbolId,
        function_block: BlockId,
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

impl ASTNode {
    pub(crate) fn is_loop(&self) -> bool {
        matches!(
            self,
            Self::For { .. } | Self::While { .. } | Self::Loop { .. }
        )
    }

    pub(crate) fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::Literal(_)
                | Self::Identifier { .. }
                | Self::MemberLookup { .. }
                | Self::Call
                | Self::BinaryOperator(_)
                | Self::UnaryOperator(_)
        )
    }

    pub(crate) fn is_statement(&self) -> bool {
        matches!(
            self,
            Self::Compound
                | Self::Return
                | Self::If
                | Self::For { .. }
                | Self::While { .. }
                | Self::Loop { .. }
                | Self::FunctionDeclaration { .. }
                | Self::VariableDeclaration { .. }
                | Self::ConstantDeclaration { .. }
                | Self::Assignment
        )
    }
}

pub struct ASTEntry {
    pub(crate) node: ASTNode,
    pub(crate) parent: Option<ASTNodeId>,
    pub(crate) first_child: Option<ASTNodeId>,
    pub(crate) last_child: Option<ASTNodeId>,
    pub(crate) next_sibling: Option<ASTNodeId>,
}

pub(crate) type ASTNodeId = usize;

pub(crate) struct ASTChildIterator<'a> {
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

pub(crate) trait ASTVisitor {
    fn accept(&self, node: &ASTNode) -> bool;
    fn visit(&mut self, node: ASTNodeId, ast: &mut AST) -> bool;
}

pub struct AST {
    root: ASTNodeId,
    entries: Vec<ASTEntry>,
}

impl Default for AST {
    fn default() -> Self {
        Self {
            root: 0,
            entries: vec![ASTEntry {
                node: ASTNode::Program,
                parent: None,
                first_child: None,
                last_child: None,
                next_sibling: None,
            }],
        }
    }
}

impl AST {
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

    pub(crate) fn get(&self, node: ASTNodeId) -> Option<&ASTNode> {
        self.entries.get(node).map(|e| &e.node)
    }

    pub(crate) fn get_mut(&mut self, node: ASTNodeId) -> Option<&mut ASTNode> {
        self.entries.get_mut(node).map(|e| &mut e.node)
    }

    fn visit_node_df(&mut self, node: ASTNodeId, visitor: &mut impl ASTVisitor) {
        let mut visit_childs = true;
        if visitor.accept(&self.entries[node].node) {
            visit_childs = visitor.visit(node, self);
        }
        if visit_childs {
            let mut current = self.entries[node].first_child;
            while let Some(child) = current {
                self.visit_node_df(child, visitor);
                current = self.entries[child].next_sibling;
            }
        }
    }

    pub(crate) fn visit_df(&mut self, visitor: &mut impl ASTVisitor) {
        self.visit_node_df(self.root(), visitor);
    }

    fn print_node(&self, node: ASTNodeId, indent: usize) {
        let entry = &self.entries[node];
        println!("{}- {:?}", " ".repeat(indent), entry.node);
        for (child, _) in self.iter_childs(node) {
            self.print_node(child, indent + 4);
        }
    }

    pub fn print(&self) {
        println!("AST:");
        self.print_node(self.root(), 0);
    }
}
