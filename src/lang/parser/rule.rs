use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::lang::lexer::token::TokenKind;

#[derive(Eq)]
pub enum Rule {
    Epsilon,
    Terminal(usize, TokenKind),
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
            Rule::Epsilon => "E".to_string(),
            Rule::Terminal(_, t) => t.repr().unwrap_or_else(|| t.name()).to_string(),
            Rule::Expandable { name, .. } => name.clone(),
            Rule::Alternative { name, .. } => name.clone(),
        }
    }

    pub fn num(&self) -> usize {
        match &self {
            Rule::Epsilon => 0,
            Rule::Terminal(num, _) => *num,
            Rule::Expandable { num, .. } => *num,
            Rule::Alternative { num, .. } => *num,
        }
    }

    pub fn sub_rules(&self) -> Option<&Vec<Rc<RefCell<Rule>>>> {
        match &self {
            Rule::Epsilon => None,
            Rule::Terminal(_, _) => None,
            Rule::Expandable { sub_rules, .. } => Some(sub_rules),
            Rule::Alternative { sub_rules, .. } => Some(sub_rules),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Rule::Terminal(_, _))
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
            Rule::Terminal(_, terminal) => terminal == token_kind,
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
            Rule::Epsilon => state.result.push_str("E"),
            Rule::Terminal(_, _) => state.result.push_str(&self.name()),
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
        self.num().hash(state)
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.num() == other.num()
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
            Rule::Epsilon => false,
            Rule::Terminal(_, _) => false,
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
            Rule::Epsilon => false,
            Rule::Terminal(_, _) => false,
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
        Rule::Epsilon => panic!("not expecting E in display itself"),
        Rule::Terminal(_, _) => panic!("not expecting terminal in display itself"),
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
        Rule::Epsilon => panic!("not expecting E in display children"),
        Rule::Terminal(_, _) => panic!("not expecting terminal in display children"),
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
        Rule::Epsilon => {}
        Rule::Terminal(_, _) => {}
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
    fn to_rule(self, num: usize) -> Rc<RefCell<Rule>>;
}

impl ToRule for TokenKind {
    fn to_rule(self, num: usize) -> Rc<RefCell<Rule>> {
        Rc::new(RefCell::new(Rule::Terminal(num, self)))
    }
}

pub fn eliminate_left_recursion(root: Rc<RefCell<Rule>>) -> Rc<RefCell<Rule>> {
    let mut by_number = BTreeMap::new();
    to_map(&root, &mut by_number);

    for (rule_num, rule) in &by_number {
        let rule = rule.borrow();
        if rule.is_terminal() {
            continue;
        }

        println!("{}", rule_num);

        loop {
            if rule.is_expandable() {
                let a_j = rule.sub_rules().unwrap().get(0).unwrap();
                if !a_j.borrow().is_terminal() && a_j.borrow().num() < rule.num() {
                    println!("working: {}", rule);
                }

                break;
            }
            else {
                for sub_rule in rule.sub_rules().unwrap() {
                    if sub_rule.borrow().is_terminal() {
                        continue;
                    }

                    let sub_rule_b = sub_rule.borrow();
                    let a_j = sub_rule_b.sub_rules().unwrap().get(0).unwrap();
                    if !a_j.borrow().is_terminal() && a_j.borrow().num() < rule.num() {
                        println!("working: {}", sub_rule.borrow());
                    }
                }

                break;
            }
        }
    }

    todo!()
}

fn to_map(rule: &Rc<RefCell<Rule>>, by_number: &mut BTreeMap<usize, Rc<RefCell<Rule>>>) {
    match &*rule.borrow() {
        Rule::Epsilon => {
            if !by_number.contains_key(&0) {
                by_number.insert(0, Rc::clone(rule));
            }
        }
        Rule::Terminal(num, _) => {
            if !by_number.contains_key(num) {
                by_number.insert(*num, Rc::clone(rule));
            }
        }
        Rule::Expandable { sub_rules, num, .. } => {
            if by_number.insert(*num, Rc::clone(rule)) == None {
                for r in sub_rules {
                    to_map(r, by_number);
                }
            }
        }
        Rule::Alternative { sub_rules, num, .. } => {
            if by_number.insert(*num, Rc::clone(rule)) == None {
                for r in sub_rules {
                    to_map(r, by_number);
                }
            }
        }
    }
}
