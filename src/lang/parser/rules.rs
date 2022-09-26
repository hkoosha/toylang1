use std::cell::Cell;
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
use crate::lang::util::extend;

pub struct Rules {
    pub rules: Vec<Rc<RefCell<Rule>>>,
    first_set: Cell<Option<HashMap<String, HashSet<TokenKind>>>>,
    follow_set: Cell<Option<HashMap<String, HashSet<TokenKind>>>>,
}

impl Rules {
    pub fn new() -> Self {
        Self::from_rules(vec![])
    }

    pub fn from_rules(rules: Vec<Rc<RefCell<Rule>>>) -> Self {
        Self {
            rules,
            first_set: Cell::new(None),
            follow_set: Cell::new(None),
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

            for alternatives in description.split('|').map(str::trim) {
                rule.borrow_mut().add_alt();
                for alt in alternatives.split(' ').map(str::trim) {
                    match TokenKind::from_repr(alt).or_else(|_| TokenKind::from_name(alt)) {
                        Ok(token_kind) => {
                            // It's a token, add it as a token.
                            rule.borrow_mut().push_last(token_kind.into());
                        },
                        Err(_) => {
                            // It's a rule.
                            if !alt.is_empty() {
                                ensure_is_valid_rule_name(alt)?;
                            }
                            let to_add = match rules.iter().find(|it| it.borrow().name() == alt) {
                                None => {
                                    // No rule already created for this name, create new
                                    let new: Rule = Rule::new(alt.to_string(), num());
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


    // =========================================================================

    fn max_recursion_elimination_num(&self) -> usize {
        get_sorted_recursion_elimination_numbers(self)
            .last()
            .map_or(0, |it| *it)
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

    fn eliminate_direct_left_recursions0(&mut self) -> bool {
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
                    new_rule.borrow_mut().push_last(TokenKind::Epsilon.into());

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

        if let Err(err) = self.validate() {
            panic!("rules are not valid: {}", err);
        }

        any_change
    }

    fn eliminate_direct_left_recursions(&mut self) {
        while self.eliminate_direct_left_recursions0() {}
    }

    // ---------------------------------

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
        self.try_find_rule_by_recursion_num(recursion_num)
            .unwrap_or_else(|| panic!("no rule with recursion num: {}", recursion_num))
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

        if let Err(err) = self.validate() {
            panic!("rules are not valid: {}", err);
        }

        any_change
    }

    pub fn eliminate_left_recursions(&mut self) {
        if let Err(err) = self.validate() {
            panic!("rules are not valid: {}", err);
        }

        while self.eliminate_indirect_left_recursions0() {}

        if let Err(err) = self.validate() {
            panic!("rules are not valid: {}", err);
        }
    }


    // =========================================================================

    pub fn validate(&self) -> Result<(), String> {
        if let Some(erroneous_rule) = self.rules.iter().find(|it| it.borrow().validate().is_err()) {
            let err_str = erroneous_rule.borrow().validate().err().unwrap();
            return Err(format!(
                "invalid rule, rule_name={} error={}",
                erroneous_rule.borrow().name(),
                err_str
            ));
        }

        // Duplicate rule.
        let mut seen = HashSet::new();
        for r in &self.rules {
            if !seen.insert(r.borrow().name().to_string()) {
                return Err(format!("duplicate rule: {}", r.borrow().name()));
            }
        }

        // Duplicate recursion elimination number.
        let numbers = get_sorted_recursion_elimination_numbers(self);
        for i in 0..numbers.len() - 1 {
            if numbers[i] == numbers[i + 1] {
                return Err(format!(
                    "duplicate recursion elimination rule: {}",
                    numbers[i]
                ));
            }
        }

        fn find_missing_rule(
            rules: &Rules,
            r: &Rc<RefCell<Rule>>,
            seen: &mut HashSet<String>,
        ) -> Result<(), String> {
            if !seen.insert(r.borrow().name().to_string()) {
                return Ok(());
            }

            for alt in &r.borrow().alternatives {
                for part in alt {
                    if part.is_rule() {
                        if !rules.has_rule(part.get_rule().borrow().name()) {
                            return Err(format!(
                                "missing rule: {}",
                                part.get_rule().borrow().name()
                            ));
                        }
                        find_missing_rule(rules, &part.get_rule(), seen)?;
                    }
                }
            }
            Ok(())
        }

        // A missing rule referenced in another rule.
        let mut seen = HashSet::new();
        for r in &self.rules {
            find_missing_rule(self, r, &mut seen)?
        }

        for r in &self.rules {
            if r.borrow().alternatives.is_empty() {
                return Err(format!(
                    "rule has has no alternative: {}",
                    r.borrow().name()
                ));
            }
            for alt in &r.borrow().alternatives {
                if alt.is_empty() {
                    return Err(format!("rule has empty alternative: {}", r.borrow().name()));
                }
            }
        }

        Ok(())
    }

    pub fn get_error(&self) -> Option<String> {
        let invalid = self.rules.iter().find(|it| it.borrow().validate().is_err());
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


    // =========================================================================

    pub fn follow_set(&self) -> HashMap<String, HashSet<TokenKind>> {
        let cache = self.follow_set.replace(None);
        match cache {
            None => {
                let calc = self.follow_set0();
                self.follow_set.replace(Some(calc.clone()));
                calc
            },
            Some(cache) => {
                let calc = cache.clone();
                self.follow_set.replace(Some(cache));
                calc
            },
        }
    }

    fn follow_set0(&self) -> HashMap<String, HashSet<TokenKind>> {
        if let Err(err) = self.validate() {
            panic!("invalid rule: {}", err);
        }

        let first = self.first_set();

        let mut follow: HashMap<String, HashSet<TokenKind>> = self
            .rules
            .iter()
            .map(|it| it.borrow().name().to_string())
            .map(|it| (it, HashSet::<TokenKind>::new()))
            .collect();

        loop {
            let mut any_change = false;

            for rule in &self.rules {
                for alt in &rule.borrow().alternatives {
                    let mut trailer = follow[rule.borrow().name()].clone();

                    for part in alt.iter().rev() {
                        if part.is_rule() {
                            let part_follow = follow.get_mut(&part.name()).unwrap();
                            any_change = any_change || extend(part_follow, trailer.clone());

                            trailer = first[&part.name()].clone();
                            trailer.remove(&TokenKind::Epsilon);
                        }
                        else {
                            trailer.clear();
                            trailer.insert(*part.get_token_kind());
                        }
                    }
                }
            }

            if !any_change {
                break;
            }
        }

        follow
    }


    pub fn first_set(&self) -> HashMap<String, HashSet<TokenKind>> {
        let cache = self.first_set.replace(None);
        match cache {
            None => {
                let calc = self.first_set0();
                self.first_set.replace(Some(calc.clone()));
                calc
            },
            Some(cache) => {
                let calc = cache.clone();
                self.first_set.replace(Some(cache));
                calc
            },
        }
    }

    fn first_set0(&self) -> HashMap<String, HashSet<TokenKind>> {
        if let Err(err) = self.validate() {
            panic!("invalid rule: {}", err);
        }

        let mut first = HashMap::new();

        for token_kind in TokenKind::values() {
            first
                .entry(token_kind.upper_name().to_string())
                .or_insert_with(HashSet::new)
                .insert(token_kind);
        }

        for rule in &self.rules {
            first.insert(rule.borrow().name().to_string(), HashSet::new());
        }

        let mut any_change = false;
        loop {
            for rule in &self.rules {
                for alt in &rule.borrow().alternatives {
                    let mut rhs: HashSet<TokenKind> = first[&alt.first().unwrap().name()]
                        .iter()
                        .filter(|it| !it.is_epsilon())
                        .cloned()
                        .collect();

                    let mut trailing = true;

                    for part_no in 0..alt.len() - 1 {
                        let part = &alt[part_no];
                        let part_first = &first[&part.name()];
                        if part_first.contains(&TokenKind::Epsilon) {
                            let next_part_first = first[&alt[part_no + 1].name()].iter().cloned();
                            rhs.extend(next_part_first);
                            rhs.remove(&TokenKind::Epsilon);
                        }
                        else {
                            trailing = false;
                            break;
                        }
                    }

                    if trailing && first[&alt.last().unwrap().name()].contains(&TokenKind::Epsilon)
                    {
                        rhs.insert(TokenKind::Epsilon);
                    }

                    let rule_first: &mut HashSet<TokenKind> =
                        first.get_mut(rule.borrow().name()).unwrap();
                    any_change = extend(rule_first, rhs);
                }
            }

            if !any_change {
                break;
            }
        }

        first
    }
}

impl PartialEq for Rules {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.rules == other.rules
    }
}

impl Eq for Rules {
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}

// FIXME worst implementation :/
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
            let stringify = r.borrow().to_string();
            let mut split = stringify.split("->");
            let name = split.next().unwrap().trim();
            let desc = split.next().unwrap().trim();
            write!(
                f,
                "\n  {: <20} -> {}",
                &name[5..].trim(),              // Remove starting 'Rules['
                &desc[..desc.len() - 1].trim(), // Remove ending ']'
            )?;
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
        .or_insert_with(|| rule.borrow().recursion_elimination_num());

    for alt in &rule.borrow().alternatives {
        for part in alt.iter().filter(|it| it.is_rule()) {
            if !numbers.contains_key(part.get_rule().borrow().name()) {
                merge_recursion_elimination_rule_to_number(&part.get_rule(), numbers);
            }
        }
    }
}

fn get_sorted_recursion_elimination_numbers(rules: &Rules) -> Vec<usize> {
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
  S                    -> fn_call | fn_declaration
  fn_call              -> ID ( args ) ;
  fn_declaration       -> FN ID ( params ) { statements }
  args                 -> arg , args | arg
  arg                  -> STRING | INT | ID
  params               -> param , params | param
  statements           -> statement , statements | statement
  param                -> ID ID
  statement            -> declaration | assignment | fn_call | ret
  declaration          -> ID ID ;
  assignment           -> ID = expressions ;
  ret                  -> RETURN expressions ;
  expressions          -> terms + expressions | terms - expressions | terms
  terms                -> factor * terms | factor / terms | factor
  factor               -> ( expressions ) | INT | ID
]
";

        EXPECTED.trim()
    }

    fn expected_recursive_grammar() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  S                    -> S fn_call | ID | S fn_declaration | RETURN
  fn_call              -> ID ( ID ) ;
  fn_declaration       -> FN ID ( S ) { fn_call }
]
";

        EXPECTED.trim()
    }

    fn expected_recursive_grammar_recursion_eliminated() -> &'static str {
        const EXPECTED: &'static str = "\
Rules[
  S                    -> ID S__0 | RETURN S__0
  fn_call              -> ID ( ID ) ;
  fn_declaration       -> FN ID ( S ) { fn_call }
  S__0                 -> fn_call S__0 | fn_declaration S__0 | EPSILON
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
  a0                   -> a1
  a1                   -> a2 ID | ID
  a2                   -> ID RETURN a2__0
  a2__0                -> ID RETURN a2__0 | EPSILON
]
        ";

        EXPECTED.trim()
    }


    #[test]
    fn test_parse() {
        let rules: Result<Rules, String> = proper_grammar().try_into();
        let rules = rules.unwrap();

        assert!(rules.validate().is_ok());
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

        println!("{}", rules);
        rules.eliminate_direct_left_recursions();

        assert!(rules.validate().is_ok());

        assert_eq!(
            rules.to_string().trim(),
            expected_recursive_grammar_recursion_eliminated()
        )
    }

    // TODO make sure the the output is correct, adjust the expected output and enable the test.
    #[test]
    fn test_eliminate_indirect_left_recursions0() {
        let rules: Result<Rules, String> = indirect_recursive_grammar0().try_into();
        let mut rules = rules.unwrap();

        let before = rules.to_string();

        rules.eliminate_left_recursions();

        assert!(rules.validate().is_ok());

        println!(
            "expected: {}",
            expected_recursive_grammar_indirect_recursion_eliminated0()
        );
        println!("before: {}", before);
        println!("after: {}", rules.to_string().trim());
        // assert_eq!(
        //     expected_recursive_grammar_indirect_recursion_eliminated0(),
        //     rules.to_string().trim(),
        // )
    }

    #[test]
    fn test_eliminate_indirect_left_recursions1() {
        let rules: Result<Rules, String> = indirect_recursive_grammar1().try_into();
        let mut rules = rules.unwrap();

        let before = rules.to_string();

        rules.eliminate_left_recursions();

        assert!(rules.validate().is_ok());

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

    #[test]
    fn test_epsilon_rule() {
        let r = "r0 -> r0 ID | EPSILON";

        let rules: Result<Rules, String> = r.try_into();
        let rules = rules.unwrap();
        println!("{}", rules.to_string());
        rules.validate().unwrap();

        assert_eq!(
            rules.to_string().trim(),
            "\
Rules[
  r0                   -> r0 ID | EPSILON
]
        "
            .trim()
            .to_string()
        )
    }

    #[test]
    fn test_first_set() {
        let r = "\
        r0 -> r0 ID | r1 | EPSILON
        r1 -> STRING
        ";

        let rules: Result<Rules, String> = r.try_into();
        let rules = rules.unwrap();
        println!("{}", rules.to_string());
        rules.validate().unwrap();

        let mut first: HashMap<String, HashSet<TokenKind>> = rules
            .first_set()
            .into_iter()
            .filter(|it| TokenKind::from_name(&it.0).is_err())
            .collect();

        println!("{:?}", first);

        assert_eq!(first.len(), 2);

        let r0 = first.remove("r0").unwrap();
        let r1 = first.remove("r1").unwrap();
        assert_eq!(r0.len(), 2);
        assert_eq!(r1.len(), 1);

        assert!(r0.contains(&TokenKind::Epsilon));
        assert!(r0.contains(&TokenKind::String));
        assert!(r1.contains(&TokenKind::String));
    }

    #[test]
    fn test_something() {
        let r = "\
        r0 -> r0 ID | r1 | r2
        r1 -> STRING
        r2 -> EPSILON
        ";

        let rules: Result<Rules, String> = r.try_into();
        let mut rules = rules.unwrap();
        rules.eliminate_left_recursions();
        println!("{}", rules.to_string());

        rules.validate().unwrap();

        let first: HashMap<String, HashSet<TokenKind>> = rules
            .first_set()
            .into_iter()
            .filter(|it| TokenKind::from_name(&it.0).is_err())
            .collect();

        let follow = rules.follow_set();

        println!("{:?}", first);
        println!("{:?}", follow);
    }
}
