use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;

use lazy_static::lazy_static;
use regex::Regex;

use crate::lang::lexer::token::TokenKind;

lazy_static! {
    static ref VALID_RULE_NAME: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}

pub(super) fn is_valid_rule_name(rule_name: &str) -> bool {
    VALID_RULE_NAME.is_match(rule_name)
}

pub(super) fn ensure_is_valid_rule_name(rule_name: &str) -> Result<&str, String> {
    if is_valid_rule_name(rule_name) {
        Ok(rule_name)
    }
    else {
        Err(format!(
            "only non-empty alphanumeric names are accepted, given name={}",
            rule_name
        ))
    }
}


#[derive(Clone, Eq)]
pub enum RulePart {
    Rule(Rc<RefCell<Rule>>),
    Token(TokenKind),
}

impl RulePart {
    pub fn is_token(&self) -> bool {
        matches!(self, RulePart::Token(_))
    }

    pub fn is_rule(&self) -> bool {
        matches!(self, RulePart::Rule(_))
    }

    pub fn get_rule(&self) -> Rc<RefCell<Rule>> {
        match self {
            RulePart::Rule(rule) => Rc::clone(rule),
            RulePart::Token(tk) => panic!("token kind is not a rule: {}", tk.repr_or_name()),
        }
    }

    pub fn get_token_kind(&self) -> &TokenKind {
        match self {
            RulePart::Rule(rule) => panic!(
                "rule is not a token kind: {}",
                rule.try_borrow()
                    .map_or_else(|_| "?".to_string(), |it| it.name.to_string())
            ),
            RulePart::Token(tk) => tk,
        }
    }

    pub fn name(&self) -> String {
        match self {
            RulePart::Rule(rule) => rule.borrow().name.to_string(),
            RulePart::Token(tk) => tk.upper_name().to_string(),
        }
    }
}

impl Display for RulePart {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            RulePart::Rule(rule) => write!(f, "RulePart::Rule[{}]", rule.borrow()),
            RulePart::Token(token_kind) => write!(f, "RulePart::Token[{}]", token_kind),
        }
    }
}

impl Debug for RulePart {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl PartialEq for RulePart {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        match self {
            RulePart::Rule(my_rule) => match other {
                RulePart::Rule(other_rule) => my_rule.borrow().name == other_rule.borrow().name,
                RulePart::Token(_) => false,
            },
            RulePart::Token(my_token_kind) => match other {
                RulePart::Rule(_) => false,
                RulePart::Token(other_token_kind) => my_token_kind == other_token_kind,
            },
        }
    }
}

impl Hash for RulePart {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.name().hash(state);
    }
}

pub fn display_of_vec_rule_part(
    rule_parts: &Vec<RulePart>,
    include_struct_name: bool,
) -> String {
    let mut display = match include_struct_name {
        true => "RuleParts<",
        false => "<",
    }
    .to_string();

    for r in rule_parts {
        display += &match r {
            RulePart::Rule(rule) => rule.borrow().name.to_string(),
            RulePart::Token(token_kind) => token_kind.upper_name().to_string(),
        };
        display += ", ";
    }

    if !rule_parts.is_empty() {
        display.pop();
        display.pop();
    }

    display += ">";
    display
}


pub struct Rule {
    name: String,
    recursion_elimination_num: usize,
    pub alternatives: Vec<Vec<RulePart>>,
}

impl Rule {
    pub(super) fn new(
        name: String,
        recursion_elimination_num: usize,
    ) -> Self {
        Self {
            name,
            recursion_elimination_num,
            alternatives: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn recursion_elimination_num(&self) -> usize {
        self.recursion_elimination_num
    }


    pub(super) fn add_alt(&mut self) {
        self.alternatives.push(vec![]);
    }

    pub(super) fn push_last(
        &mut self,
        rule_part: RulePart,
    ) {
        let len = match self.num_alts() {
            0 => panic!("no alternative exists"),
            len => len - 1,
        };
        self.push(len, rule_part)
    }

    fn push(
        &mut self,
        alt_no: usize,
        rule_part: RulePart,
    ) {
        if alt_no >= self.alternatives.len() {
            panic!("alt does not exist: {}", alt_no);
        }
        self.alternatives[alt_no].push(rule_part);
    }


    pub fn num_alts(&self) -> usize {
        self.alternatives.len()
    }

    pub fn validate(&self) -> Result<(), String> {
        let set = self
            .alternatives
            .iter()
            .map(|it| {
                it.iter()
                    .map(|it| match it {
                        RulePart::Rule(rule) => rule.borrow().name.to_string(),
                        RulePart::Token(tk) => tk.name().to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join("-")
            })
            .collect::<HashSet<_>>();

        // Duplicate rule in alternatives.
        if set.len() != self.alternatives.len() {
            let list = self
                .alternatives
                .iter()
                .map(|it| {
                    it.iter()
                        .map(|it| match it {
                            RulePart::Rule(rule) => rule.borrow().name.to_string(),
                            RulePart::Token(tk) => tk.name().to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join("-")
                })
                .filter(|it| !set.contains(it))
                .collect::<Vec<_>>();
            let thing: Vec<String> = set.iter().cloned().collect();
            return Err(format!(
                "duplicates: {} - {}",
                list.join(", "),
                thing.join(", ")
            ));
        }

        // Rule is infinitely and inherently recursive without a fix.
        // Such as the rule: some_rule -> some_rule foo bar | some_rule baz quo
        if !self.alternatives.iter().any(|it| {
            // Find any rule that does not start with recursion, if not, error.
            it.is_empty() || it[0].is_token() || it[0].get_rule().borrow().name != self.name
        }) {
            return Err(format!(
                "infinitely recursive rule: all sub-rules recurse to the same rule, self={}",
                self
            ));
        }

        // Rule has pointless sub-rule
        // Such as the rule: some_rule -> foo | some_rule
        if self.alternatives.iter().any(|it| {
            // Find any sub-rule which is single and will recurse to self, if found, error.
            it.len() == 1 && it[0].is_rule() && it[0].get_rule().borrow().name == self.name
        }) {
            return Err(format!(
                "pointless rule: a singly sub-rule refers to the same rule, self={}",
                self
            ));
        }

        if self
            .alternatives
            .iter()
            .any(|it| it.len() > 1 && it.contains(&RulePart::Token(TokenKind::Epsilon)))
        {
            return Err(format!(
                "alternative with len more than 1 contains epsilon, self={}",
                self
            ));
        }

        if self.alternatives.is_empty() {
            return Err(format!("empty rule, self={}", self));
        }

        Ok(())
    }
}


impl Drop for Rule {
    fn drop(&mut self) {
        // TODO Is this enough? or should we recurse into the list?
        self.alternatives.clear();
    }
}

impl Display for Rule {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        let alternatives = self
            .alternatives
            .iter()
            .map(|it| {
                it.iter()
                    .map(|it| match it {
                        RulePart::Rule(rule) => rule.borrow().name.to_string(),
                        RulePart::Token(tk) => tk.repr_or_name().to_uppercase(),
                    })
                    .intersperse(" ".to_string())
                    .collect::<String>()
            })
            .intersperse(" | ".to_string())
            .collect::<String>();
        write!(f, "Rule[{} -> {}]", self.name, alternatives)
    }
}

impl Hash for Rule {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.name().hash(state)
    }
}

impl PartialEq for Rule {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.name().eq(other.name())
    }
}

impl Eq for Rule {
}


impl From<TokenKind> for RulePart {
    fn from(tk: TokenKind) -> Self {
        RulePart::Token(tk)
    }
}

impl From<Rc<RefCell<Rule>>> for RulePart {
    fn from(rule: Rc<RefCell<Rule>>) -> Self {
        RulePart::Rule(rule)
    }
}

impl From<&Rc<RefCell<Rule>>> for RulePart {
    fn from(rule: &Rc<RefCell<Rule>>) -> Self {
        RulePart::Rule(Rc::clone(rule))
    }
}

impl From<Rule> for Rc<RefCell<Rule>> {
    fn from(rule: Rule) -> Self {
        Rc::new(RefCell::new(rule))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_simple() {
        let mut r0: Rule = Rule::new("r0".to_string(), 0);
        r0.add_alt();

        r0.push_last(TokenKind::Return.into());
        r0.push_last(TokenKind::Id.into());

        assert_eq!(
            format!("{}", r0),
            format!(
                "Rule[r0 -> {} {}]",
                TokenKind::Return.upper_name(),
                TokenKind::Id.upper_name()
            )
        );
    }

    #[test]
    fn test_print_circular_reference() {
        let r0: Rule = Rule::new("r0".to_string(), 0);
        let r0: Rc<RefCell<Rule>> = r0.into();

        let r1: RulePart = TokenKind::Return.into();

        let r2: Rule = Rule::new("r2".to_string(), 1);
        let r2: Rc<RefCell<Rule>> = r2.into();


        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r1);
        r0.borrow_mut().push_last(r2.clone().into());

        r2.borrow_mut().add_alt();
        r2.borrow_mut().push_last(r0.clone().into());

        assert_eq!(format!("{}", r0.borrow()), "Rule[r0 -> RETURN r2]");
        assert_eq!(format!("{}", r2.borrow()), "Rule[r2 -> r0]");
    }

    #[test]
    fn test_print_circular_self_reference() {
        let r0: Rule = Rule::new("r0".to_string(), 0);
        let r0: Rc<RefCell<Rule>> = r0.into();

        let r1: Rule = Rule::new("r1".to_string(), 1);
        let r1: Rc<RefCell<Rule>> = r1.into();

        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r1.clone().into());
        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(Rc::clone(&r0).into());
        r0.borrow_mut().push_last(r1.into());

        assert_eq!(format!("{}", r0.borrow()), "Rule[r0 -> r1 | r0 r1]");

        r0.borrow().validate().unwrap();
    }


    #[test]
    fn test_valid_circular() {
        let r0: Rule = Rule::new("r0".to_string(), 0);
        let r0: Rc<RefCell<Rule>> = r0.into();

        let r1: Rule = Rule::new("r1".to_string(), 1);
        let r1: Rc<RefCell<Rule>> = r1.into();

        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r0.clone().into());
        r0.borrow_mut().push_last(r1.clone().into());
        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r1.into());

        assert_eq!(format!("{}", r0.borrow()), "Rule[r0 -> r0 r1 | r1]");

        r0.borrow().validate().unwrap();
    }


    #[test]
    fn test_invalid_infinitely_recursive_rule_case_0() {
        let r0: Rule = Rule::new("r0".to_string(), 0);
        let r0: Rc<RefCell<Rule>> = r0.into();

        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r0.clone().into());

        assert_eq!(format!("{}", r0.borrow()), "Rule[r0 -> r0]");
        assert!(r0
            .borrow()
            .validate()
            .err()
            .unwrap()
            .starts_with("infinitely recursive rule"),);
    }

    #[test]
    fn test_invalid_infinitely_recursive_rule_case_1() {
        let r0: Rule = Rule::new("r0".to_string(), 0);
        let r0: Rc<RefCell<Rule>> = r0.into();

        let r1: Rule = Rule::new("r1".to_string(), 1);
        let r1: Rc<RefCell<Rule>> = r1.into();

        let r2: Rule = Rule::new("r2".to_string(), 2);
        let r2: Rc<RefCell<Rule>> = r2.into();

        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r0.clone().into());
        r0.borrow_mut().push_last(r1.clone().into());
        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r0.clone().into());
        r0.borrow_mut().push_last(r2.clone().into());

        assert_eq!(format!("{}", r0.borrow()), "Rule[r0 -> r0 r1 | r0 r2]");
        assert!(r0
            .borrow()
            .validate()
            .err()
            .unwrap()
            .starts_with("infinitely recursive rule"),);
    }

    #[test]
    fn test_invalid_pointless_rule() {
        let r0: Rule = Rule::new("r0".to_string(), 0);
        let r0: Rc<RefCell<Rule>> = r0.into();

        let r1: Rule = Rule::new("r1".to_string(), 1);
        let r1: Rc<RefCell<Rule>> = r1.into();

        let r2: Rule = Rule::new("r2".to_string(), 2);
        let r2: Rc<RefCell<Rule>> = r2.into();

        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r0.clone().into());
        r0.borrow_mut().push_last(r1.clone().into());
        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r0.clone().into());
        r0.borrow_mut().add_alt();
        r0.borrow_mut().push_last(r2.clone().into());

        assert_eq!(format!("{}", r0.borrow()), "Rule[r0 -> r0 r1 | r0 | r2]");
        assert!(r0
            .borrow()
            .validate()
            .err()
            .unwrap()
            .starts_with("pointless rule"),);
    }


    #[test]
    fn test_bad_rule_name() {
        assert_eq!(false, is_valid_rule_name("a b"));
        assert_eq!(false, is_valid_rule_name("a,b"));
    }
}
