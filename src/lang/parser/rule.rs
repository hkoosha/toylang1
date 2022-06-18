use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::lang::lexer::token::TokenKind;

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

impl RuleNode {
    pub fn name(&self) -> String {
        match &self {
            RuleNode::Terminal(t) => t.repr().unwrap_or_else(|| t.name()).to_string(),
            RuleNode::Expandable { name, .. } => name.clone(),
            RuleNode::Alternative { name, .. } => name.clone(),
        }
    }

    pub fn num(&self) -> Option<usize> {
        match &self {
            RuleNode::Terminal(_) => None,
            RuleNode::Expandable { num, .. } => Some(*num),
            RuleNode::Alternative { num, .. } => Some(*num),
        }
    }

    pub fn rules(&self) -> Option<&Vec<Rc<RefCell<RuleNode>>>> {
        match &self {
            RuleNode::Terminal(_) => None,
            RuleNode::Expandable { rules, .. } => Some(rules),
            RuleNode::Alternative { rules, .. } => Some(rules),
        }
    }
}

impl Drop for RuleNode {
    fn drop(&mut self) {
        let mut seen = vec![];
        erase(self, &mut seen);
    }
}

impl Display for RuleNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut state = RuleNodeDisplayState::default();

        match &self {
            RuleNode::Terminal(_) => state.result.push_str(&self.name()),
            RuleNode::Expandable { .. } => {
                do_display_itself(self, &mut state);
                do_display_children(self, &mut state);
            }
            RuleNode::Alternative { .. } => {
                do_display_itself(self, &mut state);
                do_display_children(self, &mut state);
            }
        }

        write!(f, "{}", &state.result)
    }
}

#[derive(Default)]
struct RuleNodeDisplayState {
    result: String,
    seen_ex: Vec<usize>,
    seen_al: Vec<usize>,
    seen_ex_children: Vec<usize>,
    seen_al_children: Vec<usize>,
}

impl RuleNodeDisplayState {
    fn should_display(&mut self, rule_node: &RuleNode) -> bool {
        match &rule_node {
            RuleNode::Terminal(_) => false,
            RuleNode::Expandable { num, .. } => {
                if self.seen_ex.contains(num) {
                    false
                } else {
                    self.seen_ex.push(*num);
                    true
                }
            }
            RuleNode::Alternative { num, .. } => {
                if self.seen_al.contains(num) {
                    false
                } else {
                    self.seen_al.push(*num);
                    true
                }
            }
        }
    }

    fn should_display_children(&mut self, rule_node: &RuleNode) -> bool {
        match &rule_node {
            RuleNode::Terminal(_) => false,
            RuleNode::Expandable { num, .. } => {
                if self.seen_ex_children.contains(num) {
                    false
                } else {
                    self.seen_ex_children.push(*num);
                    true
                }
            }
            RuleNode::Alternative { num, .. } => {
                if self.seen_al_children.contains(num) {
                    false
                } else {
                    self.seen_al_children.push(*num);
                    true
                }
            }
        }
    }
}

fn do_display_itself(rule_node: &RuleNode, state: &mut RuleNodeDisplayState) {
    if !state.should_display(rule_node) {
        return;
    }

    state.result.push('\n');
    match &rule_node {
        RuleNode::Terminal(_) => panic!("not expecting terminal in display itself"),
        RuleNode::Expandable { rules, name, .. } => {
            let names = rules
                .iter()
                .map(|it| it.borrow().name())
                .collect::<Vec<String>>()
                .join(" ");
            state.result.push_str(name);
            state.result.push_str(" -> ");
            state.result.push_str(&names);
        }
        RuleNode::Alternative { name, rules, .. } => {
            let names = rules
                .iter()
                .map(|it| it.borrow().name())
                .collect::<Vec<String>>()
                .join(" | ");
            state.result.push_str(name);
            state.result.push_str(" -> ");
            state.result.push_str(&names);
        }
    }
}

fn do_display_children(rule_node: &RuleNode, state: &mut RuleNodeDisplayState) {
    if !state.should_display_children(rule_node) {
        return;
    }

    match &rule_node {
        RuleNode::Terminal(_) => panic!("not expecting terminal in display children"),
        RuleNode::Expandable { rules, .. } => {
            for r in rules.iter() {
                do_display_itself(&*r.borrow(), state);
            }
            for r in rules.iter() {
                do_display_children(&*r.borrow(), state);
            }
        }
        RuleNode::Alternative { rules, .. } => {
            for r in rules.iter() {
                do_display_itself(&*r.borrow(), state);
            }
            for r in rules.iter() {
                do_display_children(&*r.borrow(), state);
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
        Rc::new(RefCell::new(RuleNode::Terminal(self)))
    }
}
