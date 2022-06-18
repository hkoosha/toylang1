use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::lang::token::TokenKind;

pub enum RuleNode {
    Terminal(TokenKind),
    Expandable {
        name: String,
        num: usize,
        rules: Vec<Rc<RefCell<RuleNode>>>,
    },
    Alternative {
        name: String,
        num: usize,
        rules: Vec<Rc<RefCell<RuleNode>>>,
    },
}

impl Drop for RuleNode {
    fn drop(&mut self) {
        let mut seen = vec![];
        erase(self, &mut seen);
    }
}

impl Display for RuleNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = "".to_string();
        let mut seen_ex = vec![];
        let mut seen_al = vec![];

        match &self {
            RuleNode::Terminal(t) => result.push_str(t.name()),
            RuleNode::Expandable { .. } => do_display(&self, &mut result, &mut seen_ex, &mut seen_al),
            RuleNode::Alternative { .. } => do_display(&self, &mut result, &mut seen_ex, &mut seen_al),
        }

        write!(f, "{}", &result[1..])
    }
}


fn do_display(rule_node: &RuleNode,
              mut result: &mut String,
              mut seen_ex: &mut Vec<usize>,
              mut seen_al: &mut Vec<usize>) {
    match &rule_node {
        RuleNode::Terminal(_) => {
            return;
        }
        RuleNode::Expandable { num, .. } => {
            if seen_ex.contains(num) {
                return;
            } else {
                seen_ex.push(*num);
            }
        }
        RuleNode::Alternative { num, .. } => {
            if seen_al.contains(num) {
                return;
            } else {
                seen_al.push(*num);
            }
        }
    }
    result.push('\n');

    match &rule_node {
        RuleNode::Terminal(_) => panic!(),
        RuleNode::Expandable { rules, name, .. } => {
            result.push_str(&name);
            result.push_str(" -> ");
            let names = rules.into_iter()
                .map(|it| {
                    match &*it.borrow() {
                        RuleNode::Terminal(t) => t.name().to_string(),
                        RuleNode::Expandable { name, .. } => name.to_string(),
                        RuleNode::Alternative { name, .. } => name.to_string(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            result.push_str(&names);
        }
        RuleNode::Alternative { name, rules, .. } => {
            result.push_str(&name);
            result.push_str(" -> ");
            let names = rules.into_iter()
                .map(|it| {
                    match &*it.borrow() {
                        RuleNode::Terminal(t) => t.name().to_string(),
                        RuleNode::Expandable { name, .. } => name.to_string(),
                        RuleNode::Alternative { .. } => panic!(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" | ");
            result.push_str(&names);
        }
    }

    match &rule_node {
        RuleNode::Terminal(_) => panic!(),
        RuleNode::Expandable { rules, .. } => {
            for r in rules.into_iter() {
                do_display(&*r.borrow(), &mut result, &mut seen_ex, &mut seen_al);
            }
        }
        RuleNode::Alternative { rules, .. } => {
            for r in rules.into_iter() {
                do_display(&*r.borrow(), &mut result, &mut seen_ex, &mut seen_al);
            }
        }
    }
}

fn erase(rule: &mut RuleNode, seen: &mut Vec<usize>) {
    match rule {
        RuleNode::Terminal(_) => {}
        RuleNode::Expandable { rules, num, .. } => {
            do_erase(rules, seen, num);
        }
        RuleNode::Alternative { rules, num, .. } => {
            do_erase(rules, seen, num);
        }
    }
}

fn do_erase(rules: &mut Vec<Rc<RefCell<RuleNode>>>, seen: &mut Vec<usize>, num: &usize) {
    if !seen.contains(num) {
        seen.push(*num);
        for sub_rule in &mut rules.iter_mut() {
            match sub_rule.try_borrow_mut() {
                Err(_) => {}
                Ok(mut ok) => {
                    erase(&mut *ok, seen);
                }
            }
        }
        rules.clear();
    }
}

pub trait ToRule {
    fn to_rule(self) -> Rc<RefCell<RuleNode>>;
}

impl ToRule for TokenKind {
    fn to_rule(self) -> Rc<RefCell<RuleNode>> {
        return Rc::new(RefCell::new(RuleNode::Terminal(self)));
    }
}