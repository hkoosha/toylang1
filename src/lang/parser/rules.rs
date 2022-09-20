use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

use crate::lang::lexer::token::TokenKind;
use crate::lang::parser::rule::ensure_is_valid_rule_name;
use crate::lang::parser::rule::Rule;
use crate::lang::parser::rule::RulePart;

#[derive(Eq, PartialEq)]
pub struct Rules {
    pub rules: Vec<Rc<RefCell<Rule>>>,
}

impl Rules {
    pub fn new() -> Self {
        Self::from_rules(vec![])
    }

    pub fn from_rules(rules: Vec<Rc<RefCell<Rule>>>) -> Self {
        Self {
            rules,
        }
    }

    pub fn parse(rules_description: &str) -> Result<Self, String> {
        let mut rules: Vec<Rc<RefCell<Rule>>> = vec![];

        let mut next_recursion_elimination_num = 0usize;
        let mut num = move || {
            let next = next_recursion_elimination_num;
            next_recursion_elimination_num += 1;
            next
        };

        for line in rules_description
            .trim()
            .lines()
            .map(str::trim)
            .filter(|it| !it.is_empty())
        {
            let mut name_to_description = line.splitn(2, "->");

            let name = {
                let name = name_to_description
                    .next()
                    .ok_or_else(|| format!("invalid rule description, missing name: {}", line))?
                    .trim();
                ensure_is_valid_rule_name(name)?;
                name
            };
            let description = {
                name_to_description
                    .next()
                    .ok_or_else(|| {
                        format!("invalid rule description, missing description: {}", line)
                    })?
                    .trim()
            };

            let rule: Rc<RefCell<Rule>> = {
                match rules
                    .iter()
                    .find(|it: &&Rc<RefCell<Rule>>| it.borrow().name() == name)
                {
                    None => {
                        // Seeing for first time.
                        let new: Rule = Rule::new(name.to_string(), num());
                        let new: Rc<RefCell<Rule>> = new.into();
                        rules.push(Rc::clone(&new));
                        new
                    },
                    Some(already) => {
                        // Seen before
                        Rc::clone(already)
                    },
                }
            };

            for alternatives in description
                .split('|')
                .map(str::trim)
                .filter(|it| !it.is_empty())
            {
                rule.borrow_mut().add_alt();
                for sub_rule in alternatives
                    .split(' ')
                    .map(str::trim)
                    .filter(|it| !it.is_empty())
                {
                    match TokenKind::from_repr(sub_rule).or_else(|_| TokenKind::from_name(sub_rule))
                    {
                        Ok(token_kind) => {
                            // It's a token, add it as a token.
                            rule.borrow_mut().push_last(token_kind.into());
                        },
                        Err(_) => {
                            // It's a rule, add it as a token.
                            ensure_is_valid_rule_name(sub_rule)?;
                            let to_add =
                                match rules.iter().find(|it| it.borrow().name() == sub_rule) {
                                    None => {
                                        // No rule already created for this name, create new
                                        let new: Rule = Rule::new(sub_rule.to_string(), num());
                                        let new: Rc<RefCell<Rule>> = new.into();
                                        rules.push(Rc::clone(&new));
                                        new
                                    },
                                    Some(already) => {
                                        // A rule already for this name exists, reuse it.
                                        Rc::clone(already)
                                    },
                                };
                            rule.borrow_mut().push_last(to_add.into());
                        },
                    }
                }
            }

            if name_to_description.next().is_some() {
                return Err(format!("invalid count of rule parts for rule: {}", line));
            }
        }

        Ok(Self::from_rules(rules))
    }


    fn eliminate_direct_left_recursions(&mut self) {
        while self.eliminate_direct_left_recursions0() {}
    }

    fn eliminate_direct_left_recursions0(&mut self) -> bool {
        if let Err(err) = self.is_valid() {
            panic!("rules are not valid: {}", err);
        }

        let mut next = self.max_recursion_elimination_num() + 1;
        let mut num = move || {
            let n = next;
            next += 1;
            n
        };

        let mut any_change = false;

        loop {
            let mut new_rule_to_add: Option<Rc<RefCell<Rule>>> = None;

            for rule in &self.rules {
                let has_any_recursive_sub_rule = has_recursive_rule(&rule.borrow());
                any_change = has_any_recursive_sub_rule;

                let name = if has_any_recursive_sub_rule {
                    Some(rule.borrow().name().to_string())
                }
                else {
                    None
                };

                if has_any_recursive_sub_rule {
                    let new_rule = {
                        let new_name = self.find_new_indexed_name(name.as_ref().unwrap().as_str());
                        let mut new_rule: Rule = Rule::new(new_name, num());
                        new_rule.add_alt();
                        let new_rule: Rc<RefCell<Rule>> = new_rule.into();
                        new_rule
                    };

                    let recursive_rules: Vec<Vec<RulePart>> = {
                        let partition_index = rule
                            .borrow_mut()
                            .alternatives
                            .iter_mut()
                            .partition_in_place(|it| {
                                !it.is_empty()
                                    && it[0].is_rule()
                                    // Risky bet: if it's borrowed, it's ourselves!
                                    && it[0].get_rule().try_borrow().map_or(true, |it| {
                                    it.name() == name.as_ref().unwrap()
                                })
                            });
                        rule.borrow_mut()
                            .alternatives
                            .drain(0..partition_index)
                            .map(|mut it| {
                                it.remove(0);
                                it.push(RulePart::Rule(Rc::clone(&new_rule)));
                                it
                            })
                            .collect()
                    };

                    new_rule.borrow_mut().alternatives = recursive_rules;
                    // epsilon rule.
                    new_rule.borrow_mut().add_alt();

                    for remaining_rule in &mut rule.borrow_mut().alternatives {
                        remaining_rule.push(RulePart::Rule(Rc::clone(&new_rule)))
                    }

                    // Goto add new rule to list and start over.
                    new_rule_to_add = Some(new_rule);
                    break;
                }
            }

            match new_rule_to_add {
                // No recursive sub-rule found anymore. Nothing more to do: any more iteration
                // yields None.
                None => break,

                // Something changed, add it and start over.
                Some(rule) => self.rules.push(rule),
            }
        }

        if let Err(err) = self.is_valid() {
            panic!("rules are not valid: {}", err);
        }

        any_change
    }

    fn find_new_indexed_name(
        &self,
        name: &str,
    ) -> String {
        for i in 0..usize::MAX {
            let new_name = format!("{}__{}", name, i);
            if !self.has_rule(&new_name) {
                return new_name;
            }
        }

        panic!("indexes exhausted for: {}", name);
    }


    fn eliminate_indirect_left_recursions0(&mut self) -> bool {
        self.eliminate_direct_left_recursions();

        let mut any_change = false;

        if let Some((i, i_alt_index, s)) = self.find_i_and_s() {
            let rule_i = self.find_rule_by_recursion_num(i);
            let mut rule_i_alt = rule_i.borrow_mut().alternatives.remove(i_alt_index);

            let recursive_call_to_rule_s = rule_i_alt.remove(0);
            let rule_s = self.find_rule_by_recursion_num(s);
            assert_eq!(recursive_call_to_rule_s.name(), rule_s.borrow().name());

            for s_alt in &rule_s.borrow().alternatives {
                let mut fix = s_alt.clone();
                fix.append(&mut rule_i_alt.clone());
                self.rules[i].borrow_mut().alternatives.push(fix);
                any_change = true;
            }
        }

        if let Err(err) = self.is_valid() {
            panic!("rules are not valid: {}", err);
        }

        any_change
    }

    fn find_i_and_s(&mut self) -> Option<(usize, usize, usize)> {
        for i in 1..=self.max_recursion_elimination_num() {
            if let Some(rule_i) = self.try_find_rule_by_recursion_num(i) {
                for s in 0..i {
                    assert_ne!(s, i);
                    if let Some(rule_s) = self.try_find_rule_by_recursion_num(s) {
                        for (rule_i_alt_num, rule_i_alt) in
                            rule_i.borrow().alternatives.iter().enumerate()
                        {
                            if !rule_i_alt.is_empty()
                                && rule_i_alt[0].is_rule()
                                && rule_i_alt[0].get_rule().borrow().name()
                                    == rule_s.borrow().name()
                            {
                                return Some((i, rule_i_alt_num, s));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn find_rule_by_recursion_num(
        &self,
        recursion_num: usize,
    ) -> Rc<RefCell<Rule>> {
        return self
            .try_find_rule_by_recursion_num(recursion_num)
            .expect(&format!("no rule with recursion num: {}", recursion_num));
    }

    fn try_find_rule_by_recursion_num(
        &self,
        recursion_num: usize,
    ) -> Option<Rc<RefCell<Rule>>> {
        for r in &self.rules {
            if r.borrow().recursion_elimination_num() == recursion_num {
                return Some(Rc::clone(r));
            }
        }

        None
    }


    pub fn eliminate_left_recursions(&mut self) {
        while self.eliminate_indirect_left_recursions0() {}
    }


    pub fn is_valid(&self) -> Result<(), String> {
        if let Some(erroneous_rule) = self.rules.iter().find(|it| it.borrow().is_valid().is_err()) {
            let err_str = erroneous_rule.borrow().is_valid().err().unwrap();
            return Err(format!(
                "invalid rule, rule_name={} error={}",
                erroneous_rule.borrow().name(),
                err_str
            ));
        }

        if self
            .rules
            .iter()
            .map(|it| it.borrow().name().to_string())
            .collect::<HashSet<_>>()
            .len()
            != self.rules.len()
        {
            return Err("duplicate rules".to_string());
        }

        let numbers = get_recursion_elimination_numbers(&self);
        for i in 0..numbers.len() - 1 {
            if numbers[i] == numbers[i + 1] {
                return Err(format!(
                    "duplicate recursion elimination rule: {}",
                    numbers[i]
                ));
            }
        }

        Ok(())
    }

    pub fn get_error(&self) -> Option<String> {
        let invalid = self.rules.iter().find(|it| !it.borrow().is_valid().is_ok());
        if invalid.is_some() {
            return Some(format!(
                "invalid rule: {}",
                invalid.unwrap().borrow().name()
            ));
        }

        let mut seen = HashSet::new();
        for i in &self.rules {
            if seen.contains(i.borrow().name()) {
                return Some(format!("duplicate rule: {}", i.borrow().name()));
            }
            seen.insert(i.borrow().name().to_string());
        }
        None
    }

    pub fn has_rule(
        &self,
        name: &str,
    ) -> bool {
        self.rules.iter().any(|it| it.borrow().name() == name)
    }


    fn max_recursion_elimination_num(&self) -> usize {
        get_recursion_elimination_numbers(self)
            .last()
            .map_or(0, |it| *it)
    }
}

impl Display for Rules {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        if self.rules.is_empty() {
            return write!(f, "Rules[]");
        }

        write!(f, "Rules[")?;
        for r in &self.rules {
            write!(f, "\n  ")?;
            write!(f, "{}", r.borrow())?;
        }
        write!(f, "\n]")
    }
}

impl TryFrom<&str> for Rules {
    type Error = String;

    fn try_from(rules_description: &str) -> Result<Self, Self::Error> {
        Self::parse(rules_description)
    }
}


fn has_recursive_rule(rule: &Rule) -> bool {
    if rule.alternatives.is_empty() {
        return false;
    }

    rule.alternatives.iter().any(|it| {
        !it.is_empty() && it[0].is_rule() && it[0].get_rule().borrow().name() == rule.name()
    })
}

fn merge_recursion_elimination_rule_to_number(
    rule: &Rc<RefCell<Rule>>,
    numbers: &mut HashMap<String, usize>,
) {
    numbers
        .entry(rule.borrow().name().to_string())
        .or_insert(rule.borrow().recursion_elimination_num());

    for alt in &rule.borrow().alternatives {
        for part in alt.iter().filter(|it| it.is_rule()) {
            if !numbers.contains_key(part.get_rule().borrow().name()) {
                merge_recursion_elimination_rule_to_number(&part.get_rule(), numbers);
            }
        }
    }
}

fn get_recursion_elimination_numbers(rules: &Rules) -> Vec<usize> {
    let mut numbers = HashMap::new();
    for r in &rules.rules {
        merge_recursion_elimination_rule_to_number(r, &mut numbers);
    }
    let mut numbers: Vec<usize> = numbers.values().into_iter().cloned().collect();
    numbers.sort();
    numbers
}


#[cfg(test)]
mod tests {
    use super::*;

    fn proper_grammar() -> &'static str {
        const GRAMMAR: &'static str = "
S               -> fn_call | fn_declaration
fn_call         -> ID ( args ) ;
args            -> arg , args | arg
arg             -> STRING | INT | ID
fn_declaration  -> FN ID ( params ) { statements }
params          -> param , params | param
param           -> ID ID
statements      -> statement , statements | statement
statement       -> declaration | assignment | fn_call | ret
declaration     -> ID ID ;
assignment      -> ID = expressions ;
expressions     -> terms + expressions | terms - expressions | terms
terms           -> factor * terms | factor / terms | factor
factor          -> ( expressions ) | INT | ID
ret             -> RETURN expressions ;
";

        GRAMMAR
    }

    fn recursive_grammar() -> &'static str {
        const GRAMMAR: &'static str = "
S               -> S fn_call | ID | S fn_declaration | RETURN
fn_call         -> ID ( ID ) ;
fn_declaration  -> FN ID ( S ) { fn_call }
";

        GRAMMAR
    }

    fn indirect_recursive_grammar0() -> &'static str {
        const GRAMMAR: &'static str = "
a0 -> a1 a2 | FN
a1 -> ID | a2 a0
a2 -> RETURN | a0 a0
";

        GRAMMAR
    }

    fn indirect_recursive_grammar1() -> &'static str {
        const GRAMMAR: &'static str = "
a0 -> a1
a1 -> a2 ID | ID
a2 -> a1 RETURN
";

        GRAMMAR
    }


    fn expected_proper_grammar() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  Rule[S -> fn_call | fn_declaration]
  Rule[fn_call -> ID ( args ) ;]
  Rule[fn_declaration -> FN ID ( params ) { statements }]
  Rule[args -> arg , args | arg]
  Rule[arg -> STRING | INTEGER | ID]
  Rule[params -> param , params | param]
  Rule[statements -> statement , statements | statement]
  Rule[param -> ID ID]
  Rule[statement -> declaration | assignment | fn_call | ret]
  Rule[declaration -> ID ID ;]
  Rule[assignment -> ID = expressions ;]
  Rule[ret -> RETURN expressions ;]
  Rule[expressions -> terms + expressions | terms - expressions | terms]
  Rule[terms -> factor * terms | factor / terms | factor]
  Rule[factor -> ( expressions ) | INTEGER | ID]
]";

        EXPECTED.trim()
    }

    fn expected_recursive_grammar() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  Rule[S -> S fn_call | ID | S fn_declaration | RETURN]
  Rule[fn_call -> ID ( ID ) ;]
  Rule[fn_declaration -> FN ID ( S ) { fn_call }]
]
";

        EXPECTED.trim()
    }

    fn expected_recursive_grammar_recursion_eliminated() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  Rule[S -> ID S__0 | RETURN S__0]
  Rule[fn_call -> ID ( ID ) ;]
  Rule[fn_declaration -> FN ID ( S ) { fn_call }]
  Rule[S__0 -> fn_call S__0 | fn_declaration S__0 | ]
]
        ";

        EXPECTED.trim()
    }

    fn expected_recursive_grammar_indirect_recursion_eliminated0() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  Rule[a0 -> a1 a2 | FN]
  Rule[a1 -> ID | a2 a0]
  Rule[a2 -> RETURN | ID a2 a0 | a a0__0 | RETURN a2 a0 a0__0]
  Rule[a0__0 -> a0 a2 a0 | a0 a2 a0 a0__0]
]
        ";

        EXPECTED.trim()
    }

    fn expected_recursive_grammar_indirect_recursion_eliminated1() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  Rule[a0 -> a1]
  Rule[a1 -> a2 ID | ID]
  Rule[a2 -> ID RETURN a2__0]
  Rule[a2__0 -> ID RETURN a2__0 | ]
]
        ";

        EXPECTED.trim()
    }


    #[test]
    fn test_parse() {
        let rules: Result<Rules, String> = proper_grammar().try_into();
        let rules = rules.unwrap();

        assert!(rules.is_valid().is_ok());
        assert_eq!(rules.to_string().trim(), expected_proper_grammar())
    }

    #[test]
    fn test_eliminate_direct_left_recursions() {
        let rules: Result<Rules, String> = recursive_grammar().try_into();
        let mut rules = rules.unwrap();

        assert_eq!(
            expected_recursive_grammar().to_string(),
            rules.to_string().trim()
        );

        rules.eliminate_direct_left_recursions();

        assert!(rules.is_valid().is_ok());

        assert_eq!(
            rules.to_string().trim(),
            expected_recursive_grammar_recursion_eliminated()
        )
    }

    // #[test]
    fn test_eliminate_indirect_left_recursions0() {
        let rules: Result<Rules, String> = indirect_recursive_grammar0().try_into();
        let mut rules = rules.unwrap();

        let before = rules.to_string();

        rules.eliminate_left_recursions();

        assert!(rules.is_valid().is_ok());

        println!(
            "expected: {}",
            expected_recursive_grammar_indirect_recursion_eliminated0()
        );
        println!("before: {}", before);
        println!("after: {}", rules.to_string().trim());
        assert_eq!(
            expected_recursive_grammar_indirect_recursion_eliminated0(),
            rules.to_string().trim(),
        )
    }

    #[test]
    fn test_eliminate_indirect_left_recursions1() {
        let rules: Result<Rules, String> = indirect_recursive_grammar1().try_into();
        let mut rules = rules.unwrap();

        let before = rules.to_string();

        rules.eliminate_left_recursions();

        assert!(rules.is_valid().is_ok());

        println!(
            "expected: {}",
            expected_recursive_grammar_indirect_recursion_eliminated1()
        );
        println!("before: {}", before);
        println!("after: {}", rules.to_string().trim());
        assert_eq!(
            expected_recursive_grammar_indirect_recursion_eliminated1(),
            rules.to_string().trim(),
        )
    }
}
