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
    pub parent: Option<Rc<RefCell<Node<'a>>>>,
    pub step_no: usize,
    pub is_focus: bool,
}

impl<'a> Node<'a> {
    pub fn root(rule: &Rc<RefCell<Rule>>) -> Rc<RefCell<Self>> {
        let this = Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
            parent: None,
            step_no: 0,
            is_focus: false,
        };
        Rc::new(RefCell::new(this))
    }

    pub fn child(
        rule: &Rc<RefCell<Rule>>,
        parent: &Rc<RefCell<Node<'a>>>,
        step_no: usize,
    ) -> Rc<RefCell<Self>> {
        let this = Self {
            rule: Rc::clone(rule),
            token: None,
            children: vec![],
            alternative_no: 0,
            parent: Some(Rc::clone(parent)),
            step_no,
            is_focus: false,
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

    pub fn alt_current_rule(&self) -> Rc<RefCell<Rule>> {
        if self.rule.borrow().is_terminal() {
            panic!("is terminal");
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

    pub fn terminal_rule(&self) -> Rc<RefCell<Rule>> {
        if self.rule.borrow().is_terminal() {
            return Rc::clone(&self.rule);
        }
        panic!("not a terminal: {}", self);
    }

    pub fn expandable_rules(&self) -> Vec<Rc<RefCell<Rule>>> {
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

pub fn ensure_parent_sane(node: &Rc<RefCell<Node>>) {
    let nb = node.borrow();

    for c in &nb.children {
        let cb = c.borrow();
        if cb.parent.is_none() {
            panic!("child has no parent, me={} child={}", nb, cb);
        }
        if !are_eq(node, cb.parent.as_ref().unwrap()) {
            panic!(
                "child does not declare me as parent,\nme={}\nchild={}\ndeclared_parent={:?}",
                nb, cb, cb.parent
            );
        }
        ensure_parent_sane(c);
    }
}

pub fn are_eq<'a>(this: &Rc<RefCell<Node<'a>>>, other: &Rc<RefCell<Node<'a>>>) -> bool {
    this.borrow().step_no == other.borrow().step_no
        && this.borrow().rule_name() == other.borrow().rule_name()
        && this.borrow().alternative_no == other.borrow().alternative_no
        && this.borrow().token.is_some() == other.borrow().token.is_some()
}

pub fn root_of<'a>(node: &Rc<RefCell<Node<'a>>>) -> Rc<RefCell<Node<'a>>> {
    if node.borrow().parent.is_some() {
        root_of(node.borrow().parent.as_ref().unwrap())
    }
    else {
        Rc::clone(node)
    }
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        result.push('\n');
        result.push_str(&self.rule_name());
        result.push('#');
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

impl<'a> Drop for Node<'a> {
    fn drop(&mut self) {
        self.parent = None;
        self.children.clear();
    }
}

fn display_node(node: &Rc<RefCell<Node>>, result: &mut String, level: usize) {
    result.push('\n');

    for _ in 0..level {
        result.push_str("  ");
    }

    result.push_str(&node.borrow().rule_name());
    result.push('#');
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
    let node1 = Node::child(&Rc::clone(&rule), &node0, 0);
    let node2 = Node::child(&Rc::clone(&rule), &node1, 0);
    let node3 = Node::child(&Rc::clone(&rule), &node2, 0);
    let node4 = Node::child(&Rc::clone(&rule), &node2, 0);
    let node5 = Node::child(&Rc::clone(&rule), &node0, 0);
    let node6 = Node::child(&Rc::clone(&rule), &node5, 0);
    let node7 = Node::child(&Rc::clone(&rule), &node5, 0);

    node2.borrow_mut().children.push(node3);
    node2.borrow_mut().children.push(node4);
    node1.borrow_mut().children.push(node2);
    node0.borrow_mut().children.push(node1);
    node5.borrow_mut().children.push(node6);
    node5.borrow_mut().children.push(node7);
    node0.borrow_mut().children.push(node5);

    println!("sample: {}", node0.borrow());
}
