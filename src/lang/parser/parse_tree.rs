use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::rule::Rule;

pub struct Node<'a> {
    rule: Rc<RefCell<Rule>>,
    pub alternative_no: usize,
    pub token: Option<Token<'a>>,
    pub children: Vec<Rc<RefCell<Node<'a>>>>,
    pub expanded: bool,
}

impl<'a> Node<'a> {
    pub fn root(rule: &Rc<RefCell<Rule>>) -> Rc<RefCell<Self>> {
        let this = Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
            expanded: false,
        };
        Rc::new(RefCell::new(this))
    }

    pub fn child(rule: &Rc<RefCell<Rule>>) -> Rc<RefCell<Self>> {
        let this = Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
            expanded: false,
        };
        Rc::new(RefCell::new(this))
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

    pub fn rule(&self) -> Rc<RefCell<Rule>> {
        if self.rule.borrow().is_terminal() || !self.rule.borrow().is_alternative() {
            return Rc::clone(&self.rule);
        }
        Rc::clone(
            self.rule
                .borrow()
                .sub_rules()
                .expect("no sub rule")
                .get(self.alternative_no)
                .expect(&format!(
                    "no such alternative={} on node={}",
                    self.alternative_no,
                    self.rule.borrow().name()
                )),
        )
    }
}

pub fn left_most_expandable<'a>(node: &Rc<RefCell<Node<'a>>>) -> Option<Rc<RefCell<Node<'a>>>> {
    if node.borrow().children.is_empty() {
        return if node.borrow().rule().borrow().is_non_terminal() {
            Some(Rc::clone(node))
        }
        else {
            None
        };
    }
    else {
        for c in &node.borrow().children {
            let node = left_most_expandable(c);
            if node.is_some() {
                return node;
            }
        }
    }

    None
}

pub fn left_most_empty_terminal<'a>(node: &Rc<RefCell<Node<'a>>>) -> Option<Rc<RefCell<Node<'a>>>> {
    if node.borrow().token.is_some() {
        return None;
    }

    if node.borrow().rule().borrow().is_terminal() {
        return Some(Rc::clone(node));
    }

    for c in &node.borrow().children {
        let node = left_most_empty_terminal(c);
        if node.is_some() {
            return node;
        }
    }

    None
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        result.push('\n');
        result.push_str(&self.rule().borrow().name());

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

fn display_node(node: &Rc<RefCell<Node>>, result: &mut String, level: usize) {
    result.push('\n');

    for _ in 0..level {
        result.push_str("  ");
    }

    result.push_str(&node.borrow().rule().borrow().name());
    if node.borrow().token.is_some() {
        result.push_str("  [");
        result.push_str(&node.borrow().token.unwrap().text);
        result.push(']');
    }

    for n in &node.borrow().children {
        display_node(n, result, level + 1);
    }
}

pub fn sample(rule: &Rc<RefCell<Rule>>) {
    let node0 = Node::root(&Rc::clone(&rule));
    let node1 = Node::child(&Rc::clone(&rule));
    let node2 = Node::child(&Rc::clone(&rule));
    let node3 = Node::child(&Rc::clone(&rule));
    let node4 = Node::child(&Rc::clone(&rule));
    let node5 = Node::child(&Rc::clone(&rule));
    let node6 = Node::child(&Rc::clone(&rule));
    let node7 = Node::child(&Rc::clone(&rule));

    node2.borrow_mut().children.push(node3);
    node2.borrow_mut().children.push(node4);
    node1.borrow_mut().children.push(node2);
    node0.borrow_mut().children.push(node1);
    node5.borrow_mut().children.push(node6);
    node5.borrow_mut().children.push(node7);
    node0.borrow_mut().children.push(node5);

    println!("sample: {}", node0.borrow());
}
