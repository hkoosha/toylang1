use log::trace;

use self::token::{Token, TokenKind};

pub mod token;

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
        iter
    }

    fn has(&self) -> bool {
        self.pos < self.text.len()
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

    fn scan_number(&mut self) -> Result<(), String> {
        while self.iter.current_char.is_some() {
            let c = self.iter.current_char.unwrap();
            if ('0'..='9').contains(&c) {
                self.add_to_buffer_and_next();
            } else if c.is_ascii_alphabetic() {
                self.is_error = true;
                return Err(format!("unexpected char while reading number: {}", c));
            } else {
                return Ok(());
            }
        }

        Ok(())
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
                Some(_) => self.add_to_buffer_and_next(),
                None => {
                    self.is_error = true;
                    return Err(format!(
                        "unterminated string at: {} => {}",
                        start,
                        &self.iter.text[self.buffer_start..]
                    ));
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
                    self.scan_number()?;
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
                    panic!(
                        "unexpected character at {}: {}",
                        self.iter.pos,
                        self.iter.current_char.unwrap()
                    );
                }
            }
        }

        if self.buffer_start != self.buffer_end {
            panic!("left over in buffer: {}", self.buffer());
        }
        Ok(None)
    }

    pub fn read_token(&mut self) -> Result<Option<Token<'a>>, String> {
        if self.is_error {
            return Err("lexer has previously encountered an error".to_string());
        }

        loop {
            match self.read_next()? {
                Some(true) => {
                    trace!(
                        "got token: {}: {}~{} = {}",
                        self.token_kind,
                        self.buffer_start,
                        self.buffer_end,
                        self.buffer()
                    );

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

impl<'a> IntoIterator for Lexer<'a> {
    type Item = Result<Token<'a>, String>;
    type IntoIter = LexerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LexerIter {
            lexer: self,
            iter_finished: false,
        }
    }
}

pub struct LexerIter<'a> {
    lexer: Lexer<'a>,
    iter_finished: bool,
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = Result<Token<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_finished {
            return None;
        }

        return match self.lexer.read_token() {
            Ok(v) => v.map(Ok),
            Err(err) => {
                self.iter_finished = true;
                Some(Err(err))
            }
        };
    }
}
