use std::fmt::{Display, Formatter};

use log::trace;

#[derive(Copy, Clone)]
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
        vec!(
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
        )
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
            _ => Err(format!("unknown token: {}", repr))
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenKind[{}]", self.name())
    }
}


pub struct Token<'a> {
    pub start_pos: usize,
    pub end_pos: usize,
    pub text: &'a str,
    pub token_kind: TokenKind,
}

impl<'a> Token<'a> {
    fn new(start_pos: usize, end_pos: usize, text: &'a str, token_kind: TokenKind) -> Self {
        Self {
            start_pos,
            end_pos,
            text,
            token_kind,
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token[{}~{} / {} / {}]", self.start_pos, self.end_pos, self.token_kind, self.text)
    }
}


struct TextCharIter<'a> {
    pos: usize,
    current_char: Option<char>,
    text: &'a str,
}

impl<'a> TextCharIter<'a> {
    fn new(text: &'a str) -> Self {
        let mut iter = Self {
            pos: 0,
            current_char: None,
            text,
        };
        iter.set();
        return iter;
    }

    fn has(&self) -> bool {
        &self.pos < &self.text.len()
    }

    fn set(&mut self) {
        if self.has() {
            self.current_char = self.text.chars().nth(self.pos)
        } else {
            self.current_char = None
        }
    }

    fn next(&mut self) {
        if self.pos <= self.text.len() {
            self.pos += 1;
        }
        self.set();
    }
}


pub struct Lexer<'a> {
    is_error: bool,
    buffer_start: usize,
    buffer_end: usize,
    in_escape: bool,
    token_kind: TokenKind,
    iter: TextCharIter<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        if text.is_empty() {
            panic!("empty text not supported");
        }

        Self {
            is_error: false,
            buffer_start: 0,
            buffer_end: 0,
            in_escape: false,
            token_kind: TokenKind::Err,
            iter: TextCharIter::new(text),
        }
    }

    fn skip_whitespaces(&mut self) {
        let mut count = 0;
        while let Some(c) = self.iter.current_char {
            match c {
                ' ' => {
                    count += 1;
                    self.iter.next();
                }
                _ => {
                    trace!("whitespaces skipped: {}", count);
                    return;
                }
            }
        }
    }

    fn add_to_buffer_and_next(&mut self) {
        self.buffer_end += 1;
        self.iter.next();
    }

    fn start_buffer(&mut self) {
        self.buffer_start = self.iter.pos;
        self.buffer_end = self.iter.pos;
    }

    fn buffer(&self) -> &'a str {
        if self.buffer_start == self.buffer_end {
            panic!("buffer is empty at: {}", self.buffer_start);
        }

        &self.iter.text[self.buffer_start..self.buffer_end]
    }

    fn scan_identifier(&mut self) {
        self.add_to_buffer_and_next();

        loop {
            match self.iter.current_char {
                Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {
                    self.add_to_buffer_and_next();
                }
                _ => {
                    self.token_kind = if let Ok(kind) = TokenKind::get_for_repr(self.buffer()) {
                        kind
                    } else {
                        TokenKind::Idt
                    };
                    return;
                }
            }
        }
    }

    fn scan_number(&mut self) {
        while self.iter.current_char.is_some()
            && '0' <= self.iter.current_char.unwrap()
            && self.iter.current_char.unwrap() <= '9' {
            self.add_to_buffer_and_next();
        }
    }

    fn scan_string(&mut self) -> Result<(), String> {
        let start = self.iter.pos;
        self.in_escape = false;
        self.iter.next();
        self.start_buffer();
        loop {
            match self.iter.current_char {
                Some('\\') => {
                    if self.in_escape {
                        self.in_escape = false;
                    } else {
                        self.in_escape = true;
                    }
                    self.add_to_buffer_and_next();
                }
                Some('"') => {
                    if self.in_escape {
                        self.add_to_buffer_and_next();
                        self.in_escape = false;
                    } else {
                        self.iter.next();
                        return Ok(());
                    }
                }
                Some(_) => {
                    self.add_to_buffer_and_next()
                }
                None => {
                    self.is_error = true;
                    return Err(format!("unterminated string at: {} => {}",
                                       start, &self.iter.text[self.buffer_start..]));
                }
            }
        }
    }

    fn read_next(&mut self) -> Result<Option<bool>, String> {
        self.start_buffer();
        if self.iter.has() {
            match self.iter.current_char.unwrap() {
                ' ' => {
                    self.skip_whitespaces();
                    return Ok(Some(false));
                }
                '\n' => {
                    self.iter.next();
                    return Ok(Some(false));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.scan_identifier();
                    return Ok(Some(true));
                }
                '0'..='9' => {
                    self.scan_number();
                    self.token_kind = TokenKind::Int;
                    return Ok(Some(true));
                }
                '"' => {
                    self.scan_string()?;
                    self.token_kind = TokenKind::Str;
                    return Ok(Some(true));
                }
                ',' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Com;
                    return Ok(Some(true));
                }
                ';' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Smi;
                    return Ok(Some(true));
                }
                '(' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Lpr;
                    return Ok(Some(true));
                }
                ')' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Rpr;
                    return Ok(Some(true));
                }
                '/' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Sls;
                    return Ok(Some(true));
                }
                '*' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Mul;
                    return Ok(Some(true));
                }
                '+' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Pls;
                    return Ok(Some(true));
                }
                '-' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Min;
                    return Ok(Some(true));
                }
                '=' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Equ;
                    return Ok(Some(true));
                }
                '{' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Lbc;
                    return Ok(Some(true));
                }
                '}' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Rbc;
                    return Ok(Some(true));
                }
                '[' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Lbr;
                    return Ok(Some(true));
                }
                ']' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Rbr;
                    return Ok(Some(true));
                }
                _ => {
                    panic!("unexpected character at {}: {}", self.iter.pos, self.iter.current_char.unwrap());
                }
            }
        }

        if self.buffer_start != self.buffer_end {
            panic!("left over in buffer: {}", self.buffer());
        }
        Ok(None)
    }

    pub fn read_token(&mut self) -> Result<Option<Token>, String> {
        if self.is_error {
            return Err("lexer has previously encountered an error".to_string());
        }

        loop {
            match self.read_next()? {
                Some(true) => {
                    trace!("got token: {}: {}~{} = {}",
                        self.token_kind, self.buffer_start, self.buffer_end, self.buffer());

                    return Ok(Some(Token::new(
                        self.buffer_start,
                        self.buffer_end,
                        self.buffer(),
                        self.token_kind,
                    )));
                }
                Some(false) => {
                    trace!("got skipper");
                    continue;
                }
                None => {
                    trace!("fin");
                    return Ok(None);
                }
            }
        }
    }
}
