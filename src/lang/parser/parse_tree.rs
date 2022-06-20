use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::rule::Rule;

pub struct Node<'a> {
    pub rule: Rc<RefCell<Rule>>,
    pub alternative_no: usize,
    pub token: Option<Token<'a>>,
    pub children: Vec<Node<'a>>,
}

impl<'a> Node<'a> {
    pub fn root(rule: &Rc<RefCell<Rule>>) -> Self {
        Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
        }
    }

    pub fn child(rule: &Rc<RefCell<Rule>>) -> Self {
        Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
        }
    }

    pub fn is_sane(&self) -> Result<(), &'static str> {
        if self.token.is_some() && !self.children.is_empty() {
            Err("intermediate nodes can not have a token")
        }
        else {
            Ok(())
        }
    }

    pub fn ensure_sane(&self) {
        if let Err(msg) = self.is_sane() {
            panic!("{}", msg);
        }
    }

    pub fn left_most_expandable(&self) -> Option<&Node> {
        if self.children.is_empty() {
            return if self.rule.borrow().is_non_terminal() {
                Some(self)
            }
            else {
                None
            };
        }
        else {
            for c in &self.children {
                let node = c.left_most_expandable();
                if node.is_some() {
                    return node;
                }
            }
        }

        None
    }

    pub fn left_most_empty_terminal(&mut self) -> Option<&mut Node<'a>> {
        if self.token.is_some() {
            return None;
        }

        if self.rule.borrow().is_terminal() {
            return Some(self);
        }

        for c in &mut self.children {
            let node = c.left_most_empty_terminal();
            if node.is_some() {
                return node;
            }
        }

        None
    }
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        result.push('\n');
        result.push_str(&self.rule.borrow().name());

        for n in &self.children {
            display_node(n, &mut result, 1);
        }

        write!(f, "{}", &result)
    }
}

impl<'a> Debug for Node<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}", self))
    }
}

fn display_node(node: &Node, result: &mut String, level: usize) {
    result.push('\n');

    for _ in 0..level {
        result.push_str("  ");
    }

    result.push_str(&node.rule.borrow().name());

    for n in &node.children {
        display_node(n, result, level + 1);
    }
}

pub fn sample(rule: &Rc<RefCell<Rule>>) {
    let mut node0 = Node::root(&Rc::clone(&rule));
    let mut node1 = Node::child(&Rc::clone(&rule));
    let mut node2 = Node::child(&Rc::clone(&rule));
    let node3 = Node::child(&Rc::clone(&rule));
    let node4 = Node::child(&Rc::clone(&rule));
    let mut node5 = Node::child(&Rc::clone(&rule));
    let node6 = Node::child(&Rc::clone(&rule));
    let node7 = Node::child(&Rc::clone(&rule));

    node2.children.push(node3);
    node2.children.push(node4);
    node1.children.push(node2);
    node0.children.push(node1);
    node5.children.push(node6);
    node5.children.push(node7);
    node0.children.push(node5);

    println!("sample: {}", node0);
}
