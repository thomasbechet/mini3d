use super::token::TokenKind;

#[derive(Debug, PartialEq)]
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
            Self::Equal
            | Self::NotEqual
            | Self::LessEqual
            | Self::GreaterEqual
            | Self::Less
            | Self::Greater
            | Self::And
            | Self::Or => 3,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        matches!(
            self,
            Self::Addition | Self::Subtraction | Self::Multiplication | Self::Division
        )
    }
}

#[derive(Debug, PartialEq)]
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
