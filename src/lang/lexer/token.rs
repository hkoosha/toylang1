use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub enum TokenKind {
    Error,
    Eof,
    Epsilon,
    Id,
    Fn,
    Return,
    Int,
    String,
    LeftParen,
    RightParen,
    LeftBraces,
    RightBraces,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Equal,
    Slash,
    Star,
    Minus,
    Plus,
}

impl TokenKind {
    pub fn values() -> Vec<Self> {
        [
            Self::Error,
            Self::Eof,
            Self::Epsilon,
            Self::Id,
            Self::Fn,
            Self::Return,
            Self::Int,
            Self::String,
            Self::LeftParen,
            Self::RightParen,
            Self::LeftBraces,
            Self::RightBraces,
            Self::LeftBracket,
            Self::RightBracket,
            Self::Semicolon,
            Self::Comma,
            Self::Equal,
            Self::Slash,
            Self::Star,
            Self::Minus,
            Self::Plus,
        ]
        .to_vec()
    }

    pub fn from_repr(repr: &str) -> Result<Self, String> {
        match repr {
            "fn" => Ok(Self::Fn),
            "return" => Ok(Self::Return),
            "(" => Ok(Self::LeftParen),
            ")" => Ok(Self::RightParen),
            "[" => Ok(Self::LeftBracket),
            "]" => Ok(Self::RightBracket),
            "{" => Ok(Self::LeftBraces),
            "}" => Ok(Self::RightBraces),
            ";" => Ok(Self::Semicolon),
            "," => Ok(Self::Comma),
            "=" => Ok(Self::Equal),
            "/" => Ok(Self::Slash),
            "*" => Ok(Self::Star),
            "-" => Ok(Self::Minus),
            "+" => Ok(Self::Plus),
            _ => Err(format!("unknown TokenKind representation: {}", repr)),
        }
    }

    pub fn from_repr_including_epsilon(repr: &str) -> Result<Self, String> {
        match repr {
            "" => Ok(Self::Epsilon),
            "fn" => Ok(Self::Fn),
            "return" => Ok(Self::Return),
            "(" => Ok(Self::LeftParen),
            ")" => Ok(Self::RightParen),
            "[" => Ok(Self::LeftBracket),
            "]" => Ok(Self::RightBracket),
            "{" => Ok(Self::LeftBraces),
            "}" => Ok(Self::RightBraces),
            ";" => Ok(Self::Semicolon),
            "," => Ok(Self::Comma),
            "=" => Ok(Self::Equal),
            "/" => Ok(Self::Slash),
            "*" => Ok(Self::Star),
            "-" => Ok(Self::Minus),
            "+" => Ok(Self::Plus),
            _ => Err(format!("unknown TokenKind representation: {}", repr)),
        }
    }

    //noinspection SpellCheckingInspection
    pub fn from_name(repr: &str) -> Result<Self, String> {
        match repr.to_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "eof" => Ok(Self::Eof),
            "epsilon" => Ok(Self::Epsilon),
            "id" => Ok(Self::Id),
            "fn" => Ok(Self::Fn),
            "return" => Ok(Self::Return),
            "int" => Ok(Self::Int),
            "string" => Ok(Self::String),
            "left_paren" => Ok(Self::LeftParen),
            "right_paren" => Ok(Self::RightParen),
            "left_braces" => Ok(Self::LeftBraces),
            "right_braces" => Ok(Self::RightBraces),
            "left_brackets" => Ok(Self::LeftBracket),
            "right_brackets" => Ok(Self::RightBracket),
            "semicolon" => Ok(Self::Semicolon),
            "comma" => Ok(Self::Comma),
            "equal" => Ok(Self::Equal),
            "slash" => Ok(Self::Slash),
            "star" => Ok(Self::Star),
            "minus" => Ok(Self::Minus),
            "plus" => Ok(Self::Plus),
            _ => Err(format!("unknown TokenKind name: {}", repr)),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Eof => "eof",
            Self::Epsilon => "epsilon",
            Self::Id => "id",
            Self::Fn => "fn",
            Self::Return => "return",
            Self::Int => "integer",
            Self::String => "string",
            Self::LeftParen => "left_paren",
            Self::RightParen => "right_paren",
            Self::LeftBraces => "lef_braces",
            Self::RightBraces => "right_braces",
            Self::LeftBracket => "left_brackets",
            Self::RightBracket => "right_brackets",
            Self::Semicolon => "semicolon",
            Self::Comma => "comma",
            Self::Equal => "equal",
            Self::Slash => "slash",
            Self::Star => "star",
            Self::Minus => "minus",
            Self::Plus => "plus",
        }
    }

    pub fn upper_name(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Eof => "EOF",
            Self::Epsilon => "EPSILON",
            Self::Id => "ID",
            Self::Fn => "FN",
            Self::Return => "RETURN",
            Self::Int => "INT",
            Self::String => "STRING",
            Self::LeftParen => "LEFT_PAREN",
            Self::RightParen => "RIGHT_PAREN",
            Self::LeftBraces => "LEFT_BRACES",
            Self::RightBraces => "RIGHT_BRACES",
            Self::LeftBracket => "LEFT_BRACKETS",
            Self::RightBracket => "RIGHT_BRACKETS",
            Self::Semicolon => "SEMICOLON",
            Self::Comma => "COMMA",
            Self::Equal => "EQUAL",
            Self::Slash => "SLASH",
            Self::Star => "STAR",
            Self::Minus => "MINUS",
            Self::Plus => "PLUS",
        }
    }

    pub fn repr(&self) -> Option<&'static str> {
        match self {
            Self::Fn => Some("fn"),
            Self::Return => Some("return"),
            Self::LeftParen => Some("("),
            Self::RightParen => Some(")"),
            Self::LeftBraces => Some("{"),
            Self::RightBraces => Some("}"),
            Self::LeftBracket => Some("["),
            Self::RightBracket => Some("]"),
            Self::Semicolon => Some(";"),
            Self::Comma => Some(","),
            Self::Equal => Some("="),
            Self::Slash => Some("/"),
            Self::Star => Some("*"),
            Self::Minus => Some("-"),
            Self::Plus => Some("+"),
            _ => None,
        }
    }

    pub fn repr_or_name(&self) -> &'static str {
        match self.repr() {
            None => self.upper_name(),
            Some(repr) => repr,
        }
    }

    pub fn is_keyword(&self) -> bool {
        self.repr().is_some()
    }

    pub fn is_epsilon(&self) -> bool {
        *self == Self::Epsilon
    }

    pub fn is_eof(&self) -> bool {
        *self == Self::Eof
    }
}

impl Display for TokenKind {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "TokenKind[{}]", self.name())
    }
}

// =============================================================================

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub start_pos: usize,
    pub end_pos: usize,
    pub line: usize,
    pub text: &'a str,
    pub token_kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn new(
        start_pos: usize,
        end_pos: usize,
        line: usize,
        text: &'a str,
        token_kind: TokenKind,
    ) -> Self {
        Self {
            start_pos,
            end_pos,
            line,
            text,
            token_kind,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "Token[L{} / {}~{}-{} / {}]",
            self.line, self.start_pos, self.end_pos, self.token_kind, self.text
        )
    }
}
