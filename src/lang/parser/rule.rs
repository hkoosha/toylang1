use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::lang::lexer::token::TokenKind;

#[derive(Eq)]
pub enum Rule {
    Terminal(TokenKind),
    Expandable {
        name: String,
        num: usize,
        sub_rules: Vec<Rc<RefCell<Rule>>>,
    },
    Alternative {
        name: String,
        num: usize,
        sub_rules: Vec<Rc<RefCell<Rule>>>,
    },
}

impl Rule {
    pub fn name(&self) -> String {
        match &self {
            Rule::Terminal(t) => t.repr().unwrap_or_else(|| t.name()).to_string(),
            Rule::Expandable { name, .. } => name.clone(),
            Rule::Alternative { name, .. } => name.clone(),
        }
    }

    pub fn num(&self) -> Option<usize> {
        match &self {
            Rule::Terminal(_) => None,
            Rule::Expandable { num, .. } => Some(*num),
            Rule::Alternative { num, .. } => Some(*num),
        }
    }

    pub fn sub_rules(&self) -> Option<&Vec<Rc<RefCell<Rule>>>> {
        match &self {
            Rule::Terminal(_) => None,
            Rule::Expandable { sub_rules, .. } => Some(sub_rules),
            Rule::Alternative { sub_rules, .. } => Some(sub_rules),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Rule::Terminal(_))
    }

    pub fn is_alternative(&self) -> bool {
        matches!(self, Rule::Alternative { .. })
    }

    pub fn is_expandable(&self) -> bool {
        matches!(self, Rule::Expandable { .. })
    }

    pub fn is_non_terminal(&self) -> bool {
        !self.is_terminal()
    }

    pub fn matches(&self, token_kind: &TokenKind) -> bool {
        match self {
            Rule::Terminal(terminal) => terminal == token_kind,
            _ => panic!("expecting a terminal"),
        }
    }

    pub fn has_next(&self, alt: usize) -> bool {
        return self.sub_rules().is_some() && self.sub_rules().unwrap().get(alt).is_some();
    }
}

impl Drop for Rule {
    fn drop(&mut self) {
        let mut seen = vec![];
        erase(self, &mut seen);
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut state = RuleDisplayState::default();

        match &self {
            Rule::Terminal(_) => state.result.push_str(&self.name()),
            Rule::Expandable { .. } => {
                do_display_itself(self, &mut state);
                do_display_children(self, &mut state);
            }
            Rule::Alternative { .. } => {
                do_display_itself(self, &mut state);
                do_display_children(self, &mut state);
            }
        }

        write!(f, "{}", &state.result)
    }
}

impl Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            Rule::Terminal(t) => {
                t.hash(state);
            }
            Rule::Expandable { name, num, .. } => {
                state.write_usize(*num);
                name.hash(state);
                "expandable".hash(state);
            }
            Rule::Alternative { name, num, .. } => {
                state.write_usize(*num);
                name.hash(state);
                "alternative".hash(state);
            }
        }
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Rule::Terminal(st) => match other {
                Rule::Terminal(ot) => *st == *ot,
                _ => false,
            },
            Rule::Expandable { name, num, .. } => match other {
                Rule::Terminal(_) => false,
                Rule::Expandable {
                    name: o_name,
                    num: o_num,
                    ..
                } => *name == *o_name && *num == *o_num,
                Rule::Alternative { .. } => false,
            },
            Rule::Alternative { name, num, .. } => match other {
                Rule::Terminal(_) => false,
                Rule::Expandable { .. } => false,
                Rule::Alternative {
                    name: o_name,
                    num: o_num,
                    ..
                } => *name == *o_name && *num == *o_num,
            },
        }
    }
}

impl Clone for Rule {
    fn clone(&self) -> Self {
        match self {
            Rule::Terminal(t) => Self::Terminal(*t),
            Rule::Expandable {
                name,
                num,
                sub_rules,
            } => Self::Expandable {
                name: name.clone(),
                num: *num,
                sub_rules: sub_rules.clone(),
            },
            Rule::Alternative {
                name,
                num,
                sub_rules,
            } => Self::Alternative {
                name: name.clone(),
                num: *num,
                sub_rules: sub_rules.clone(),
            },
        }
    }
}

#[derive(Default)]
struct RuleDisplayState {
    result: String,
    seen_ex: Vec<usize>,
    seen_al: Vec<usize>,
    seen_ex_children: Vec<usize>,
    seen_al_children: Vec<usize>,
}

impl RuleDisplayState {
    fn should_display(&mut self, rule_node: &Rule) -> bool {
        match &rule_node {
            Rule::Terminal(_) => false,
            Rule::Expandable { num, .. } => {
                if self.seen_ex.contains(num) {
                    false
                }
                else {
                    self.seen_ex.push(*num);
                    true
                }
            }
            Rule::Alternative { num, .. } => {
                if self.seen_al.contains(num) {
                    false
                }
                else {
                    self.seen_al.push(*num);
                    true
                }
            }
        }
    }

    fn should_display_children(&mut self, rule_node: &Rule) -> bool {
        match &rule_node {
            Rule::Terminal(_) => false,
            Rule::Expandable { num, .. } => {
                if self.seen_ex_children.contains(num) {
                    false
                }
                else {
                    self.seen_ex_children.push(*num);
                    true
                }
            }
            Rule::Alternative { num, .. } => {
                if self.seen_al_children.contains(num) {
                    false
                }
                else {
                    self.seen_al_children.push(*num);
                    true
                }
            }
        }
    }
}

fn do_display_itself(rule_node: &Rule, state: &mut RuleDisplayState) {
    if !state.should_display(rule_node) {
        return;
    }

    state.result.push('\n');
    match &rule_node {
        Rule::Terminal(_) => panic!("not expecting terminal in display itself"),
        Rule::Expandable {
            sub_rules: rules,
            name,
            ..
        } => {
            let names = rules
                .iter()
                .map(|it| it.borrow().name())
                .collect::<Vec<String>>()
                .join(" ");
            state.result.push_str(&format!("{:15}", name));
            state.result.push_str(" -> ");
            state.result.push_str(&names);
        }
        Rule::Alternative {
            name,
            sub_rules: rules,
            ..
        } => {
            let names = rules
                .iter()
                .map(|it| it.borrow().name())
                .collect::<Vec<String>>()
                .join(" | ");
            state.result.push_str(&format!("{:15}", name));
            state.result.push_str(" -> ");
            state.result.push_str(&names);
        }
    }
}

fn do_display_children(rule_node: &Rule, state: &mut RuleDisplayState) {
    if !state.should_display_children(rule_node) {
        return;
    }

    match &rule_node {
        Rule::Terminal(_) => panic!("not expecting terminal in display children"),
        Rule::Expandable {
            sub_rules: rules, ..
        } => {
            for r in rules.iter() {
                do_display_itself(&*r.borrow(), state);
            }
            for r in rules.iter() {
                do_display_children(&*r.borrow(), state);
            }
        }
        Rule::Alternative {
            sub_rules: rules, ..
        } => {
            for r in rules.iter() {
                do_display_itself(&*r.borrow(), state);
            }
            for r in rules.iter() {
                do_display_children(&*r.borrow(), state);
            }
        }
    }
}

fn erase(rule: &mut Rule, seen: &mut Vec<usize>) {
    match rule {
        Rule::Terminal(_) => {}
        Rule::Expandable {
            sub_rules: rules,
            num,
            ..
        } => {
            do_erase(rules, seen, num);
        }
        Rule::Alternative {
            sub_rules: rules,
            num,
            ..
        } => {
            do_erase(rules, seen, num);
        }
    }
}

fn do_erase(rules: &mut Vec<Rc<RefCell<Rule>>>, seen: &mut Vec<usize>, num: &usize) {
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
    fn to_rule(self) -> Rc<RefCell<Rule>>;
}

impl ToRule for TokenKind {
    fn to_rule(self) -> Rc<RefCell<Rule>> {
        Rc::new(RefCell::new(Rule::Terminal(self)))
    }
}

pub fn eliminate_left_recursion(root: Rc<RefCell<Rule>>) -> Rc<RefCell<Rule>> {
    let mut map = HashMap::new();
    let mut numbers = HashMap::new();
    to_map(&root, &mut map, &mut numbers, &mut 0);

    for (name, rule) in &map {
        if rule.is_terminal() {
            continue;
        }

        let rule_num = numbers[name];

        loop {
            if rule.is_expandable() {
                let first = rule.sub_rules().unwrap().get(0).unwrap();
                if first.borrow().is_non_terminal() && numbers[&first.borrow().name()] < rule_num {
                    panic!("kill me");
                }
            }
            else {
                for sub_rule in rule.sub_rules().unwrap() {
                    match &*sub_rule.borrow() {
                        Rule::Terminal(_) => {}
                        Rule::Expandable { .. } => {}
                        Rule::Alternative { .. } => {}
                    }
                }
            }
        }
    }
    todo!()
}

fn to_map(
    rule: &Rc<RefCell<Rule>>,
    map: &mut HashMap<String, Rule>,
    numbers: &mut HashMap<String, usize>,
    carry: &mut usize,
) {
    match &*rule.borrow() {
        Rule::Terminal(t) => {
            if !numbers.contains_key(t.name()) {
                map.insert(t.name().to_string(), rule.borrow().clone());
                numbers.insert(t.name().to_string(), *carry);
                *carry += 1;
            }
        }
        Rule::Expandable {
            name, sub_rules, ..
        } => {
            if map.insert(name.clone(), rule.borrow().clone()) == None {
                numbers.insert(name.clone(), *carry);
                *carry += 1;
                for r in sub_rules {
                    to_map(r, map, numbers, carry);
                }
            }
        }
        Rule::Alternative {
            name, sub_rules, ..
        } => {
            if map.insert(name.clone(), rule.borrow().clone()) == None {
                numbers.insert(name.clone(), *carry);
                *carry += 1;
                for r in sub_rules {
                    to_map(r, map, numbers, carry);
                }
            }
        }
    }
}
