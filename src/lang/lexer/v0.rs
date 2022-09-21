use log::trace;

use crate::lang::lexer::token::Token;
use crate::lang::lexer::token::TokenKind;

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
        }
        else {
            self.current_char = None
        }
    }

    fn next(&mut self) {
        self.pos += 1;
        self.set();
    }
}

impl<'a> From<&'a str> for TextCharIter<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text)
    }
}

// =============================================================================

pub struct Lexer<'a> {
    is_error: bool,
    buffer_start: usize,
    buffer_end: usize,
    current_line: usize,
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
            current_line: 1,
            in_escape: false,
            token_kind: TokenKind::Error,
            iter: text.into(),
        }
    }

    fn skip_whitespaces(&mut self) {
        let mut count = 0;
        while let Some(c) = self.iter.current_char {
            match c {
                ' ' => {
                    count += 1;
                    self.iter.next();
                },
                _ => {
                    trace!("whitespaces skipped: {}", count);
                    return;
                },
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

    fn scan(&mut self) {
        self.add_to_buffer_and_next();

        loop {
            match self.iter.current_char {
                Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {
                    self.add_to_buffer_and_next();
                },
                _ => {
                    self.token_kind = if let Ok(kind) = TokenKind::from_repr(self.buffer()) {
                        kind
                    }
                    else {
                        TokenKind::Id
                    };
                    return;
                },
            }
        }
    }

    fn scan_number(&mut self) -> Result<(), String> {
        while self.iter.current_char.is_some() {
            let c = self.iter.current_char.unwrap();
            if ('0'..='9').contains(&c) {
                self.add_to_buffer_and_next();
            }
            else if c.is_ascii_alphabetic() {
                self.is_error = true;
                return Err(format!(
                    "unexpected char while reading number, line={} char={}",
                    self.current_line, c
                ));
            }
            else {
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
                Some('"') => {
                    if self.in_escape {
                        self.add_to_buffer_and_next();
                        self.in_escape = false;
                    }
                    else {
                        self.iter.next();
                        return Ok(());
                    }
                },
                Some('\\') => {
                    self.in_escape = !self.in_escape;
                    self.add_to_buffer_and_next();
                },
                Some('\n') => {
                    self.current_line += 1;
                    self.in_escape = false;
                    self.add_to_buffer_and_next()
                },
                Some(_) => {
                    self.in_escape = false;
                    self.add_to_buffer_and_next()
                },
                None => {
                    self.is_error = true;
                    return Err(format!(
                        "unterminated string at: {} => {}",
                        start,
                        &self.iter.text[self.buffer_start..]
                    ));
                },
            }
        }
    }

    fn read_next(&mut self) -> Result<Option<bool>, String> {
        self.start_buffer();

        if self.iter.has() {
            return match self.iter.current_char.unwrap() {
                ' ' => {
                    self.skip_whitespaces();
                    Ok(Some(false))
                },
                '\n' => {
                    self.current_line += 1;
                    self.iter.next();
                    Ok(Some(false))
                },
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.scan();
                    Ok(Some(true))
                },
                '0'..='9' => {
                    self.scan_number()?;
                    self.token_kind = TokenKind::Integer;
                    Ok(Some(true))
                },
                '"' => {
                    self.scan_string()?;
                    self.token_kind = TokenKind::String;
                    Ok(Some(true))
                },
                ',' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Comma;
                    Ok(Some(true))
                },
                ';' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Semicolon;
                    Ok(Some(true))
                },
                '(' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::LeftParen;
                    Ok(Some(true))
                },
                ')' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::RightParen;
                    Ok(Some(true))
                },
                '/' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Slash;
                    Ok(Some(true))
                },
                '*' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Star;
                    Ok(Some(true))
                },
                '+' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Plus;
                    Ok(Some(true))
                },
                '-' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Minus;
                    Ok(Some(true))
                },
                '=' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::Equal;
                    Ok(Some(true))
                },
                '{' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::LeftBraces;
                    Ok(Some(true))
                },
                '}' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::RightBraces;
                    Ok(Some(true))
                },
                '[' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::LeftBracket;
                    Ok(Some(true))
                },
                ']' => {
                    self.add_to_buffer_and_next();
                    self.token_kind = TokenKind::RightBracket;
                    Ok(Some(true))
                },
                _ => Err(format!(
                    "unexpected character at line={} pos={}: {}",
                    self.current_line,
                    self.iter.pos,
                    self.iter.current_char.unwrap()
                )),
            };
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

                    return Ok(Some(Token {
                        start_pos: self.buffer_start,
                        end_pos: self.buffer_end,
                        line: self.current_line,
                        text: self.buffer(),
                        token_kind: self.token_kind,
                    }));
                },
                Some(false) => {
                    trace!("got skipper");
                    continue;
                },
                None => {
                    trace!("fin");
                    return Ok(None);
                },
            }
        }
    }
}

impl<'a> From<&'a str> for Lexer<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text)
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
            Ok(v) => {
                if v.is_none() {
                    self.iter_finished = true;
                }
                v.map(Ok)
            },
            Err(err) => {
                self.iter_finished = true;
                Some(Err(err))
            },
        };
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::lang::lexer::token::TokenKind;

    #[test]
    fn test_id0() {
        let lexer: Lexer = "hello\nwhatever".into();
        let mut i = 0;
        for x in lexer.into_iter() {
            let x = x.unwrap();
            match i {
                0 => {
                    assert_eq!(x.token_kind, TokenKind::Id);
                    assert_eq!(x.line, 1);
                },
                1 => {
                    assert_eq!(x.token_kind, TokenKind::Id);
                    assert_eq!(x.line, 2);
                },
                _ => panic!(),
            }
            i += 1;
        }
    }

    #[test]
    fn test_fn_and_id() {
        let lexer: Lexer = "fn my_thing42".into();
        let mut i = 0;
        for x in lexer.into_iter() {
            let x = x.unwrap();
            match i {
                0 => {
                    assert_eq!(x.token_kind, TokenKind::Fun);
                    assert_eq!(x.line, 1);
                },
                1 => {
                    assert_eq!(x.token_kind, TokenKind::Id);
                    assert_eq!(x.line, 1);
                },
                _ => panic!(),
            }
            i += 1;
        }
    }
}
