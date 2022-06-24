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
    pub step_no: usize,
}

impl<'a> Node<'a> {
    pub fn root(rule: &Rc<RefCell<Rule>>) -> Rc<RefCell<Self>> {
        let this = Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
            expanded: false,
            step_no: 0,
        };
        Rc::new(RefCell::new(this))
    }

    pub fn child(rule: &Rc<RefCell<Rule>>, step_no: usize) -> Rc<RefCell<Self>> {
        let this = Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
            expanded: false,
            step_no,
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
        if self.rule.borrow().is_terminal() {
            return Rc::clone(&self.rule);
        }
        if !self.rule.borrow().is_alternative() {
            panic!("is expandable");
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

    pub fn rules(&self) -> Vec<Rc<RefCell<Rule>>> {
        match &*self.rule.borrow() {
            Rule::Terminal(_) => panic!("is terminal, not expandable"),
            Rule::Alternative { .. } => panic!("is alternative, not expandable"),
            Rule::Expandable { sub_rules, .. } => sub_rules.clone(),
        }
    }

    pub fn rule_name(&self) -> String {
        match &*self.rule.borrow() {
            Rule::Terminal(t) => t.repr().unwrap_or_else(|| t.name()).to_string(),
            Rule::Alternative { name, .. } => name.clone(),
            Rule::Expandable { name, .. } => name.clone(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        match &*self.rule.borrow() {
            Rule::Terminal(_) => true,
            _ => false,
        }
    }

    pub fn is_expandable(&self) -> bool {
        match &*self.rule.borrow() {
            Rule::Expandable { .. } => true,
            _ => false,
        }
    }

    pub fn is_alternative(&self) -> bool {
        match &*self.rule.borrow() {
            Rule::Alternative { .. } => true,
            _ => false,
        }
    }

    pub fn has_next_alt(&self) -> bool {
        if !self.rule.borrow().is_alternative() {
            false
        }
        else {
            self.rule
                .borrow()
                .sub_rules()
                .expect("no sub rule")
                .get(self.alternative_no + 1)
                .is_some()
        }
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

    if node.borrow().is_terminal() {
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
        result.push_str(&self.rule_name());
        result.push('@');
        result.push_str(&self.step_no.to_string());

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

    result.push_str(&node.borrow().rule_name());
    result.push('@');
    result.push_str(&node.borrow().step_no.to_string());
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
    let node1 = Node::child(&Rc::clone(&rule), 0);
    let node2 = Node::child(&Rc::clone(&rule), 0);
    let node3 = Node::child(&Rc::clone(&rule), 0);
    let node4 = Node::child(&Rc::clone(&rule), 0);
    let node5 = Node::child(&Rc::clone(&rule), 0);
    let node6 = Node::child(&Rc::clone(&rule), 0);
    let node7 = Node::child(&Rc::clone(&rule), 0);

    node2.borrow_mut().children.push(node3);
    node2.borrow_mut().children.push(node4);
    node1.borrow_mut().children.push(node2);
    node0.borrow_mut().children.push(node1);
    node5.borrow_mut().children.push(node6);
    node5.borrow_mut().children.push(node7);
    node0.borrow_mut().children.push(node5);

    println!("sample: {}", node0.borrow());
}
