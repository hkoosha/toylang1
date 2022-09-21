use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::rule::RulePart;

pub struct Node<'a> {
    pub rule_part: RulePart,
    pub alt_no: Option<usize>,

    pub token: Option<Token<'a>>,

    pub parent: Option<Rc<RefCell<Node<'a>>>>,
    pub children: Vec<Rc<RefCell<Node<'a>>>>,

    num: usize,
}

impl Node<'_> {
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
}

impl<'a> Into<Rc<RefCell<Node<'a>>>> for Node<'a> {
    fn into(self) -> Rc<RefCell<Node<'a>>> {
        Rc::new(RefCell::new(self))
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
