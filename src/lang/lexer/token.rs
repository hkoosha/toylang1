use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Err,
    Eof,
    Idt,
    Fun,
    Ret,
    Int,
    Str,
    Lpr,
    Rpr,
    Lbr,
    Rbr,
    Lbc,
    Rbc,
    Smi,
    Com,
    Equ,
    Sls,
    Mul,
    Min,
    Pls,
}

impl TokenKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenKind::Err => "ERR",
            TokenKind::Eof => "EOF",
            TokenKind::Idt => "IDT",
            TokenKind::Fun => "FUN",
            TokenKind::Ret => "RET",
            TokenKind::Int => "INT",
            TokenKind::Str => "STR",
            TokenKind::Lpr => "LPR",
            TokenKind::Rpr => "RPR",
            TokenKind::Lbr => "LBR",
            TokenKind::Rbr => "RBR",
            TokenKind::Lbc => "LBC",
            TokenKind::Rbc => "RBC",
            TokenKind::Smi => "SMI",
            TokenKind::Com => "COM",
            TokenKind::Equ => "EQU",
            TokenKind::Sls => "SLS",
            TokenKind::Mul => "MUL",
            TokenKind::Min => "MIN",
            TokenKind::Pls => "PLS",
        }
    }

    pub fn repr(&self) -> Option<&'static str> {
        match self {
            TokenKind::Fun => Some("fn"),
            TokenKind::Ret => Some("return"),
            TokenKind::Lpr => Some("("),
            TokenKind::Rpr => Some(")"),
            TokenKind::Lbr => Some("["),
            TokenKind::Rbr => Some("]"),
            TokenKind::Lbc => Some("{"),
            TokenKind::Rbc => Some("}"),
            TokenKind::Smi => Some(";"),
            TokenKind::Com => Some(","),
            TokenKind::Equ => Some("="),
            TokenKind::Sls => Some("/"),
            TokenKind::Mul => Some("*"),
            TokenKind::Min => Some("-"),
            TokenKind::Pls => Some("+"),
            _ => None,
        }
    }

    pub fn is_keyword(&self) -> bool {
        self.repr().is_some()
    }

    pub fn all() -> Vec<TokenKind> {
        vec![
            Self::Err,
            Self::Eof,
            Self::Idt,
            Self::Fun,
            Self::Ret,
            Self::Int,
            Self::Str,
            Self::Lpr,
            Self::Rpr,
            Self::Lbr,
            Self::Rbr,
            Self::Lbc,
            Self::Rbc,
            Self::Smi,
            Self::Com,
            Self::Equ,
            Self::Sls,
            Self::Mul,
            Self::Min,
            Self::Pls,
        ]
    }

    pub fn get_for_repr(repr: &str) -> Result<TokenKind, String> {
        match repr {
            "fn" => Ok(Self::Fun),
            "return" => Ok(Self::Ret),
            "(" => Ok(Self::Lpr),
            ")" => Ok(Self::Rpr),
            "[" => Ok(Self::Lbr),
            "]" => Ok(Self::Rbr),
            "{" => Ok(Self::Lbc),
            "}" => Ok(Self::Rbc),
            ";" => Ok(Self::Smi),
            "," => Ok(Self::Com),
            "=" => Ok(Self::Equ),
            "/" => Ok(Self::Sls),
            "*" => Ok(Self::Mul),
            "-" => Ok(Self::Min),
            "+" => Ok(Self::Pls),
            _ => Err(format!("unknown token: {}", repr)),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenKind[{}]", self.name())
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub start_pos: usize,
    pub end_pos: usize,
    pub text: &'a str,
    pub token_kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn new(start_pos: usize, end_pos: usize, text: &'a str, token_kind: TokenKind) -> Self {
        Self {
            start_pos,
            end_pos,
            text,
            token_kind,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token[{}~{} / {} / {}]",
            self.start_pos, self.end_pos, self.token_kind, self.text
        )
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token[{}~{} / {} / {}]",
            self.start_pos, self.end_pos, self.token_kind, self.text
        )
    }
}
