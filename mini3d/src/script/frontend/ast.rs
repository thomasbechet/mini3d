use anyhow::{Result, anyhow, Context};

use super::lexer::TokenKind;

#[derive(Debug)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    And,
    Or,
}

impl From<TokenKind> for BinaryOperator {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Plus => Self::Addition,
            TokenKind::Minus => Self::Subtraction,
            TokenKind::Multiply => Self::Multiplication,
            TokenKind::Divide => Self::Division,
            TokenKind::Equal => Self::Equal,
            TokenKind::NotEqual => Self::NotEqual,
            TokenKind::LessEqual => Self::LessEqual,
            TokenKind::GreaterEqual => Self::GreaterEqual,
            TokenKind::Less => Self::Less,
            TokenKind::Greater => Self::Greater,
            TokenKind::And => Self::And,
            TokenKind::Or => Self::Or,
            _ => panic!("Invalid token kind for binary operator"),
        }
    }
}

impl BinaryOperator {

    pub fn precedence(&self) -> u32 {
        match self {
            Self::Addition | Self::Subtraction => 1,
            Self::Multiplication | Self::Division => 2,
            Self::Equal | Self::NotEqual | Self::LessEqual | Self::GreaterEqual | 
            Self::Less | Self::Greater | Self::And | Self::Or => 3,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        matches!(self, Self::Addition | Self::Subtraction | Self::Multiplication | Self::Division)
    }
}

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
    Not,
}

impl From<TokenKind> for UnaryOperator {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Minus => Self::Minus,
            TokenKind::Not => Self::Not,
            _ => panic!("Invalid token kind for unary operator"),
        }
    }
}

#[derive(Debug)]
pub enum Literal<'a> {
    Integer(i32),
    Float(f32),
    String(&'a str),
    Boolean(bool),
    Nil,
}

#[derive(Debug)]
pub enum ASTPrimitive {
    Boolean,
    Integer,
    Float,
    String,
    Entity,
    Object,
}

impl ASTPrimitive {

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "bool" => Ok(Self::Boolean),
            "int" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "string" => Ok(Self::String),
            "entity" => Ok(Self::Entity),
            "object" => Ok(Self::Object),
            _ => Err(anyhow!("Invalid primitive type")),
        }
    }
}

#[derive(Debug)]
pub enum ASTNode<'a> {
    Program,
    Import { path: &'a str, identifier: &'a str, },
    CompoundStatement, // STMT-0, STMT-1, STMT-2, ...
    Literal(Literal<'a>),
    Identifier(&'a str),
    MemberLookup(&'a str), // PARENT-0
    ReturnStatement, // EXPR
    IfStatement, // CONDITION-0, BODY-0, CONDITION-1, BODY-1, ...
    IfBody,
    ForStatement, // IDENTIFIER-0, GENERATOR-0, BODY-0
    ForBody,
    CommentStatement(&'a str),
    FunctionDeclaration { identifier: &'a str, return_type: Option<ASTPrimitive>, }, // ARG-0, ARG-1, ..., COMPOUNT-STMT
    FunctionArgument { identifier: &'a str, arg_type: Option<ASTPrimitive>, },
    VariableDeclaration { identifier: &'a str, var_type: Option<ASTPrimitive>, }, // EXPR
    Call,
    Assignment, // IDENTIFIER-0, EXPR-0
    BinaryOperator(BinaryOperator), // LEFT-EXPR-0, RIGHT-EXPR-1
    UnaryOperator(UnaryOperator), // EXPR
}

pub struct ASTEntry<'a> {
    pub(crate) node: ASTNode<'a>,
    pub(crate) parent: Option<ASTNodeId>,
    pub(crate) first_child: Option<ASTNodeId>,
    pub(crate) last_child: Option<ASTNodeId>,
    pub(crate) next_sibling: Option<ASTNodeId>,
}

pub(crate) type ASTNodeId = usize;

pub struct ASTChildIterator<'a> {
    next: Option<ASTNodeId>,
    ast: &'a AST<'a>,
}

impl<'a> Iterator for ASTChildIterator<'a> {
    type Item = (ASTNodeId, &'a ASTNode<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next {
            self.next = self.ast.entries[next].next_sibling;
            Some((next, &self.ast.entries[next].node))
        } else {
            None
        }
    }
}

pub struct AST<'a> {
    root: ASTNodeId,
    entries: Vec<ASTEntry<'a>>,
}

impl<'a> AST<'a> {

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
        }
    }

    pub(crate) fn root(&self) -> ASTNodeId {
        self.root
    }

    pub(crate) fn add(&mut self, node: ASTNode<'a>) -> ASTNodeId {
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

    pub(crate) fn append_child(&mut self, parent: ASTNodeId, child: ASTNodeId) -> Result<()> {
        if let Some(last_child) = self.entries[parent].last_child {
            self.entries[last_child].next_sibling = Some(child);
            self.entries[parent].last_child = Some(child);
        } else {
            self.entries[parent].first_child = Some(child);
            self.entries[parent].last_child = Some(child);
        }
        self.entries[child].parent = Some(parent);
        Ok(())
    }

    pub(crate) fn iter_childs(&'a self, node: ASTNodeId) -> ASTChildIterator<'a> {
        let next = self.entries[node].first_child;
        ASTChildIterator { next, ast: self }
    }

    pub(crate) fn get_mut(&mut self, node: ASTNodeId) -> Result<&mut ASTNode<'a>> {
        Ok(&mut self.entries.get_mut(node).with_context(|| "Invalid node id")?.node)
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