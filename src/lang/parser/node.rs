use std::cell::RefCell;
use std::cmp::max;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::rule::RulePart;

pub struct Node<'a> {
    rule_part: RulePart,
    alt_no: Option<usize>,

    token: Option<Token<'a>>,

    parent: Option<Rc<RefCell<Node<'a>>>>,
    children: Vec<Rc<RefCell<Node<'a>>>>,

    num: usize,
}

impl<'a> Node<'a> {
    pub fn new(
        rule_part: RulePart,
        num: usize,
    ) -> Self {
        let mut node = Self {
            rule_part,
            alt_no: None,
            token: None,
            parent: None,
            children: vec![],
            num,
        };

        if node.rule_part.is_rule() && node.has_next_alt() {
            node.next_alt();
        }

        node
    }

    pub fn new_with_parent(
        rule_part: RulePart,
        num: usize,
        parent: &Rc<RefCell<Node<'a>>>,
    ) -> Self {
        let mut node = Self {
            rule_part,
            alt_no: None,
            token: None,
            parent: Some(Rc::clone(parent)),
            children: vec![],
            num,
        };

        if node.rule_part.is_rule() && node.has_next_alt() {
            node.next_alt();
        }

        node
    }


    pub fn has_next_alt(&self) -> bool {
        if !self.rule_part.is_rule() {
            panic!("node is not a rule: {}", self.rule_part.name());
        }

        match self.alt_no {
            None => !self.rule_part.get_rule().borrow().alternatives.is_empty(),
            Some(alt_no) => (alt_no + 1) < self.rule_part.get_rule().borrow().alternatives.len(),
        }
    }

    pub fn next_alt(&mut self) {
        if !self.rule_part.is_rule() {
            panic!("node is not a rule");
        }

        if !self.has_next_alt() {
            panic!(
                "node has no more alt, current alt: {}, rule: {}",
                self.alt_no.unwrap_or(0),
                self.rule_part
            );
        }

        self.alt_no = Some(match self.alt_no {
            None => 0,
            Some(alt_no) => alt_no + 1,
        });
    }


    pub fn has_alt(&self) -> bool {
        if !self.rule_part.is_rule() {
            panic!("node is not a rule");
        }

        match self.alt_no {
            None => {
                if self.rule_part.get_rule().borrow().alternatives.is_empty() {
                    false
                }
                else {
                    unreachable!("next_alt() not called on construction, this is a bug")
                }
            },
            Some(_) => true,
        }
    }

    pub fn alt(&self) -> usize {
        if !self.rule_part.is_rule() {
            panic!("node is not a rule");
        }

        match self.alt_no {
            None => {
                if self.rule_part.get_rule().borrow().alternatives.is_empty() {
                    panic!(
                        "no alternative available on: {}",
                        self.rule_part.get_rule().borrow().name()
                    )
                }
                else {
                    unreachable!("next_alt() not called on construction, this is a bug")
                }
            },
            Some(alt_no) => alt_no,
        }
    }


    pub fn num(&self) -> usize {
        self.num
    }

    pub fn next_num(&self) -> usize {
        if let Some(parent) = self.parent.as_ref() {
            return parent.borrow().next_num();
        }

        self.max_num() + 1
    }

    fn max_num(&self) -> usize {
        let mut next_num = self.num();

        for r in &self.children {
            next_num = max(next_num, r.borrow().max_num())
        }

        next_num
    }


    pub fn rule_part(&self) -> &RulePart {
        &self.rule_part
    }

    pub fn parent(&self) -> &Option<Rc<RefCell<Node<'a>>>> {
        &self.parent
    }


    pub fn token(&self) -> &Option<Token<'a>> {
        &self.token
    }

    pub fn drain_token(&mut self) -> Token<'a> {
        let mut drain: Option<Token<'a>> = None;
        std::mem::swap(&mut drain, &mut self.token);
        drain.unwrap()
    }

    pub fn set_token(
        &mut self,
        t: Token<'a>,
    ) {
        self.token = Some(t);
    }

    pub fn children(&self) -> &Vec<Rc<RefCell<Node<'a>>>> {
        &self.children
    }

    pub fn set_children(
        &mut self,
        children: Vec<Rc<RefCell<Node<'a>>>>,
    ) {
        self.children = children
    }

    pub fn append_child(
        &mut self,
        child: &Rc<RefCell<Node<'a>>>,
    ) {
        self.children.push(Rc::clone(child));
    }
}

impl Drop for Node<'_> {
    fn drop(&mut self) {
        self.parent = None;
        // TODO is this needed?
        // TODO is this enough? should we recurse?
        self.children.clear();
    }
}

impl<'a> From<Node<'a>> for Rc<RefCell<Node<'a>>> {
    fn from(node: Node<'a>) -> Self {
        Rc::new(RefCell::new(node))
    }
}


pub fn display_of(node: &Rc<RefCell<Node<'_>>>) -> String {
    let mut display = String::new();
    display_of0(node, &mut display, 0);
    display
}

fn display_of0(
    node: &Rc<RefCell<Node<'_>>>,
    display: &mut String,
    level: usize,
) {
    if level > 0 {
        display.push('\n');
        display.push('|');
        let multiplier = match level {
            1 => 1,
            _ => 2,
        };
        for _ in 0..level * multiplier {
            display.push('_');
        }
        display.push(' ');
    }
    display.push_str(&node.borrow().rule_part.name());
    if node.borrow().token.is_some() {
        display.push('[');
        display.push_str(node.borrow().token.as_ref().unwrap().text);
        display.push(']');
    }
    for child in &node.borrow().children {
        display_of0(child, display, level + 1);
    }
}


pub type ParseResult<'a> = Result<Rc<RefCell<Node<'a>>>, ParseError<'a>>;

pub struct ParseError<'a> {
    partial_tree: Rc<RefCell<Node<'a>>>,
    error: String,
}

impl<'a> ParseError<'a> {
    pub fn new(
        partial_tree: &Rc<RefCell<Node<'a>>>,
        error: String,
    ) -> Self {
        Self {
            partial_tree: Rc::clone(partial_tree),
            error,
        }
    }

    pub fn error(&self) -> &str {
        &self.error
    }

    pub fn partial_tree(&self) -> &Rc<RefCell<Node<'a>>> {
        &self.partial_tree
    }
}

impl Debug for ParseError<'_> {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "ParseError[{}]", self.error)
    }
}

impl Display for ParseError<'_> {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "ParseError[{}]", self.error)
    }
}
