use crate::script::{
    frontend::error::{CompileError, LexicalError},
    mir::primitive::PrimitiveType,
};

use super::{
    literal::Literal,
    strings::{StringId, StringTable},
    token::{Location, Span, Token, TokenKind, TokenValue},
};

pub(crate) struct Lexer {
    peeks: Vec<Token>,
    buffer: String,
    char_peek: Option<(char, Location)>,
    parse_comments: bool,
}

fn is_identifier_character(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Lexer {
    pub(crate) fn new(parse_comments: bool) -> Self {
        Self {
            peeks: Vec::new(),
            buffer: String::new(),
            char_peek: None,
            parse_comments,
        }
    }

    fn flush_buffer(&mut self, strings: &mut StringTable) -> StringId {
        let id = strings.add(&self.buffer);
        self.buffer.clear();
        id
    }

    fn peek_char(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
    ) -> Option<(char, Location)> {
        if self.char_peek.is_some() {
            self.char_peek
        } else {
            self.char_peek = self.next_char(chars);
            self.char_peek
        }
    }

    fn next_char(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
    ) -> Option<(char, Location)> {
        if let Some(c) = self.char_peek {
            self.char_peek = None;
            Some(c)
        } else {
            chars.next()
        }
    }

    fn consume_comment(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        strings: &mut StringTable,
    ) -> Result<Option<Token>, CompileError> {
        let start_location = self.peek_char(chars).unwrap().1;
        if self.parse_comments {
            let mut stop_location = start_location;
            while let Some((c, loc)) = self.peek_char(chars) {
                self.buffer.push(c);
                if loc.line != start_location.line {
                    break;
                }
                self.next_char(chars).unwrap();
                stop_location = loc;
            }
            let id = self.flush_buffer(strings);
            Ok(Some(Token {
                kind: TokenKind::Comment,
                span: Span {
                    start: start_location,
                    stop: stop_location,
                },
                value: TokenValue::Comment(id),
            }))
        } else {
            let previous_line = start_location.line;
            while let Some((_, loc)) = self.peek_char(chars) {
                if loc.line != previous_line {
                    break;
                }
                self.next_char(chars).unwrap();
            }
            Ok(None)
        }
    }

    fn consume_string(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        strings: &mut StringTable,
    ) -> Result<Token, CompileError> {
        let start_location = self.next_char(chars).unwrap().1;
        let mut stop_location = start_location;
        while let Some((c, loc)) = self.next_char(chars) {
            stop_location = loc;
            if c == '\'' {
                // Check for end of string
                let span = Span {
                    start: start_location,
                    stop: stop_location,
                };
                let id = self.flush_buffer(strings);
                return Ok(Token {
                    kind: TokenKind::Literal,
                    span,
                    value: TokenValue::Literal(Literal::String(id)),
                });
            }
            self.buffer.push(c);
        }
        Err(LexicalError::UnterminatedString {
            span: Span {
                start: start_location,
                stop: stop_location,
            },
        }
        .into())
    }

    fn consume_identifier(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        strings: &mut StringTable,
    ) -> Result<Token, CompileError> {
        let start_location = self.peek_char(chars).unwrap().1;
        let mut stop_location = start_location;
        while let Some((c, loc)) = self.peek_char(chars) {
            if !is_identifier_character(c) {
                break;
            }
            stop_location = loc;
            self.next_char(chars).unwrap();
            self.buffer.push(c);
        }

        // Check if identifier is a keyword
        let keyword = match self.buffer.as_str() {
            "import" => Some(TokenKind::Import),
            "export" => Some(TokenKind::Export),
            "let" => Some(TokenKind::Let),
            "const" => Some(TokenKind::Const),
            "if" => Some(TokenKind::If),
            "then" => Some(TokenKind::Then),
            "else" => Some(TokenKind::Else),
            "elif" => Some(TokenKind::Elif),
            "end" => Some(TokenKind::End),
            "do" => Some(TokenKind::Do),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "while" => Some(TokenKind::While),
            "function" => Some(TokenKind::Function),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "return" => Some(TokenKind::Return),
            "not" => Some(TokenKind::Not),
            "as" => Some(TokenKind::As),
            _ => None,
        };
        let span = Span {
            start: start_location,
            stop: stop_location,
        };
        // Check if identifier is a keyword
        if let Some(kind) = keyword {
            // We can flush the buffer as its value is not needed
            self.buffer.clear();
            Ok(Token {
                kind,
                span,
                value: TokenValue::None,
            })
        } else {
            // Check if identifier is a literal
            let literal = match self.buffer.as_str() {
                "true" => Some(Literal::Boolean(true)),
                "false" => Some(Literal::Boolean(false)),
                "nil" => Some(Literal::Nil),
                _ => None,
            };
            if let Some(literal) = literal {
                self.buffer.clear();
                Ok(Token {
                    kind: TokenKind::Literal,
                    span,
                    value: TokenValue::Literal(literal),
                })
            } else {
                // Check if identifier is a primitive type
                let primitive = match self.buffer.as_str() {
                    "bool" => Some(PrimitiveType::Boolean),
                    "int" => Some(PrimitiveType::Integer),
                    "float" => Some(PrimitiveType::Float),
                    "vec2" => Some(PrimitiveType::Vec2),
                    "ivec2" => Some(PrimitiveType::IVec2),
                    "vec3" => Some(PrimitiveType::Vec3),
                    "ivec3" => Some(PrimitiveType::IVec3),
                    "vec4" => Some(PrimitiveType::Vec4),
                    "ivec4" => Some(PrimitiveType::IVec4),
                    "mat4" => Some(PrimitiveType::Mat4),
                    "quat" => Some(PrimitiveType::Quat),
                    "string" => Some(PrimitiveType::String),
                    "entity" => Some(PrimitiveType::Entity),
                    "object" => Some(PrimitiveType::Object),
                    _ => None,
                };
                if let Some(primitive) = primitive {
                    self.buffer.clear();
                    Ok(Token {
                        kind: TokenKind::PrimitiveType,
                        span,
                        value: TokenValue::PrimitiveType(primitive),
                    })
                } else {
                    let id = self.flush_buffer(strings);
                    Ok(Token {
                        kind: TokenKind::Identifier,
                        span,
                        value: TokenValue::Identifier(id),
                    })
                }
            }
        }
    }

    fn consume_number(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
    ) -> Result<Token, CompileError> {
        let start_location = self.peek_char(chars).unwrap().1;
        let mut stop_location = start_location;
        let mut has_dot = false;
        while let Some((c, loc)) = self.peek_char(chars) {
            if c == '.' || c.is_ascii_digit() {
                self.buffer.push(c);
                stop_location = loc;
                // Check for float dot
                if c == '.' {
                    if has_dot {
                        return Err(LexicalError::MalformedNumber {
                            span: Span {
                                start: start_location,
                                stop: stop_location,
                            },
                        }
                        .into());
                    } else {
                        has_dot = true;
                    }
                }
                // Consume character and append to buffer
                self.next_char(chars).unwrap();
            } else if is_identifier_character(c) {
                return Err(LexicalError::MalformedNumber {
                    span: Span {
                        start: start_location,
                        stop: stop_location,
                    },
                }
                .into());
            } else {
                break;
            }
        }
        let span = Span {
            start: start_location,
            stop: stop_location,
        };
        if has_dot {
            let value = self
                .buffer
                .as_str()
                .parse()
                .map_err(|error| LexicalError::FloatParseError { span, error })?;
            self.buffer.clear();
            Ok(Token {
                kind: TokenKind::Literal,
                span,
                value: TokenValue::Literal(Literal::Float(value)),
            })
        } else {
            let value = self
                .buffer
                .as_str()
                .parse()
                .map_err(|error| LexicalError::IntegerParseError { span, error })?;
            self.buffer.clear();
            Ok(Token {
                kind: TokenKind::Literal,
                span,
                value: TokenValue::Literal(Literal::Integer(value)),
            })
        }
    }

    fn consume_spaces(&mut self, chars: &mut impl Iterator<Item = (char, Location)>) {
        while let Some((c, _)) = self.peek_char(chars) {
            if !c.is_whitespace() {
                break;
            }
            self.next_char(chars);
        }
    }

    fn consume_single_char_token(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        kind: TokenKind,
        loc: Location,
    ) -> Result<Token, CompileError> {
        self.next_char(chars).unwrap();
        Ok(Token::single(kind, loc))
    }

    fn parse_token(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        strings: &mut StringTable,
    ) -> Result<Token, CompileError> {
        while let Some((c, loc)) = self.peek_char(chars) {
            match c {
                '+' => return self.consume_single_char_token(chars, TokenKind::Plus, loc),
                '-' => {
                    self.next_char(chars).unwrap();
                    if let Some((next, _)) = self.peek_char(chars) {
                        if next == '-' {
                            // Comment detected
                            self.next_char(chars).unwrap(); // Skip second '-' character
                            let comment = self.consume_comment(chars, strings)?;
                            if self.parse_comments {
                                return Ok(comment.unwrap());
                            }
                        } else {
                            return Ok(Token::single(TokenKind::Minus, loc));
                        }
                    } else {
                        return Ok(Token::single(TokenKind::Minus, loc));
                    }
                }
                '*' => return self.consume_single_char_token(chars, TokenKind::Multiply, loc),
                '/' => return self.consume_single_char_token(chars, TokenKind::Divide, loc),
                '(' => return self.consume_single_char_token(chars, TokenKind::LeftParen, loc),
                ')' => return self.consume_single_char_token(chars, TokenKind::RightParen, loc),
                '[' => return self.consume_single_char_token(chars, TokenKind::LeftBracket, loc),
                ']' => return self.consume_single_char_token(chars, TokenKind::RightBracket, loc),
                '{' => return self.consume_single_char_token(chars, TokenKind::LeftBrace, loc),
                '}' => return self.consume_single_char_token(chars, TokenKind::RightBrace, loc),
                ',' => return self.consume_single_char_token(chars, TokenKind::Comma, loc),
                ':' => return self.consume_single_char_token(chars, TokenKind::Colon, loc),
                '.' => return self.consume_single_char_token(chars, TokenKind::Dot, loc),
                '=' => {
                    self.next_char(chars).unwrap();
                    if let Some((next, _)) = self.peek_char(chars) {
                        if next == '=' {
                            // Double equal detected
                            self.next_char(chars).unwrap(); // Skip second '=' character
                            return Ok(Token::double(TokenKind::Equal, loc));
                        }
                    }
                    return Ok(Token::single(TokenKind::Assign, loc));
                }
                '<' => {
                    self.next_char(chars).unwrap();
                    if let Some((next, _)) = self.peek_char(chars) {
                        if next == '=' {
                            self.next_char(chars).unwrap();
                            return Ok(Token::double(TokenKind::LessEqual, loc));
                        }
                    }
                    return Ok(Token::single(TokenKind::Less, loc));
                }
                '>' => {
                    self.next_char(chars).unwrap();
                    if let Some((next, _)) = self.peek_char(chars) {
                        if next == '=' {
                            self.next_char(chars).unwrap();
                            return Ok(Token::double(TokenKind::GreaterEqual, loc));
                        }
                    }
                    return Ok(Token::single(TokenKind::Greater, loc));
                }
                '!' => {
                    self.next_char(chars).unwrap();
                    if let Some((next, _)) = self.peek_char(chars) {
                        if next == '=' {
                            self.next_char(chars).unwrap();
                            return Ok(Token::double(TokenKind::NotEqual, loc));
                        }
                    }
                    return Err(LexicalError::IllegalCharacter {
                        span: Span {
                            start: loc,
                            stop: loc,
                        },
                        c,
                    }
                    .into());
                }
                '\'' => return self.consume_string(chars, strings),
                ' ' => {
                    self.consume_spaces(chars); // Ignore spaces
                }
                _ => {
                    if c.is_numeric() {
                        // Try to parse a number
                        return self.consume_number(chars);
                    } else if is_identifier_character(c) {
                        // Try to parse an identifier
                        return self.consume_identifier(chars, strings);
                    } else {
                        // Simply consume the character
                        self.next_char(chars).unwrap();
                    }
                }
            }
        }
        Ok(Token::eof())
    }

    pub(crate) fn peek(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        strings: &mut StringTable,
        lookahead: usize,
    ) -> Result<Token, CompileError> {
        while self.peeks.len() <= lookahead {
            let token = self.parse_token(chars, strings)?;
            self.peeks.push(token);
        }
        Ok(self.peeks[lookahead])
    }

    pub(crate) fn next(
        &mut self,
        chars: &mut impl Iterator<Item = (char, Location)>,
        strings: &mut StringTable,
    ) -> Result<Token, CompileError> {
        if self.peeks.is_empty() {
            self.parse_token(chars, strings)
        } else {
            Ok(self.peeks.remove(0))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::script::frontend::source::stream::SourceStream;
    #[test]
    fn test_multiple_lines() {
        let mut script = SourceStream::new("let x = 2 + 2\nlet y = 3 + 3");
        let mut lexer = Lexer::new(false);
        let mut strings = StringTable::default();
        for _ in 0..2 {
            let token = lexer.next(&mut script, &mut strings).unwrap();
            assert_eq!(token.kind, TokenKind::Let);
            let token = lexer.next(&mut script, &mut strings).unwrap();
            assert_eq!(token.kind, TokenKind::Identifier);
            let token = lexer.next(&mut script, &mut strings).unwrap();
            assert_eq!(token.kind, TokenKind::Assign);
            let token = lexer.next(&mut script, &mut strings).unwrap();
            assert_eq!(token.kind, TokenKind::Literal);
            let token = lexer.next(&mut script, &mut strings).unwrap();
            assert_eq!(token.kind, TokenKind::Plus);
            let token = lexer.next(&mut script, &mut strings).unwrap();
            assert_eq!(token.kind, TokenKind::Literal);
        }
    }

    #[test]
    fn test_eof() {
        let mut script = SourceStream::new("end");
        let mut lexer = Lexer::new(false);
        let mut strings = StringTable::default();
        let token = lexer.next(&mut script, &mut strings).unwrap();
        assert_eq!(token.kind, TokenKind::End);
        let token = lexer.next(&mut script, &mut strings).unwrap();
        assert_eq!(token.kind, TokenKind::EOF);
        let token = lexer.next(&mut script, &mut strings).unwrap();
        assert_eq!(token.kind, TokenKind::EOF);
    }
}
