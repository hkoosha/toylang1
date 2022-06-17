use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::marker::PhantomPinned;
use std::rc::Rc;

use crate::lang::token::TokenKind;

pub mod inefficient_parser;

enum Rule {
    Terminal(TokenKind),
    Expandable {
        name: String,
        num: u32,
        rules: Vec<Rc<RefCell<Rule>>>,
        _pin: PhantomPinned,
    },
}

impl Drop for Rule {
    fn drop(&mut self) {
        let mut seen = vec![];
        erase(self, &mut seen);
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut seen = vec![];
        let mut result = "".to_string();
        print_rule(self, &mut seen, &mut result, 0);
        write!(f, "{}", result)
    }
}

fn erase(rule: &mut Rule, seen: &mut Vec<u32>) {
    match rule {
        Rule::Terminal(_) => {}
        Rule::Expandable { rules, num, .. } => {
            if !seen.contains(num) {
                seen.push(*num);
                for sub_rule in &mut rules.into_iter() {
                    erase(&mut sub_rule.borrow_mut(), seen);
                }
                rules.clear();
            }
        }
    }
}

fn print_rule(rule: &Rule, seen: &mut Vec<u32>, result: &mut String, level: usize) {
    result.push('\n');
    for _ in 0..level {
        result.push_str("  ");
    }

    match rule {
        Rule::Terminal(t) => {
            result.push_str(t.name());
        }
        Rule::Expandable { rules, num, name, .. } => {
            result.push_str(&name);
            if !seen.contains(num) {
                seen.push(*num);
                for sub_rule in &mut rules.into_iter() {
                    print_rule(&*sub_rule.borrow(), seen, result, level + 1);
                }
            } else {
                result.push('*');
            }
        }
    }
}
