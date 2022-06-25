use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::parse_tree::{left_most_empty_terminal, Node};
use crate::lang::parser::rule::Rule;

struct BacktrackingParser<'a> {
    tokens: Vec<Token<'a>>,
    tree: Rc<RefCell<Node<'a>>>,
    step_no: isize,
}

impl<'a> BacktrackingParser<'a> {
    fn expand(&mut self) -> bool {
        if is_matching_ready(&self.tree) == MatchingReady::Neutral {
            return false;
        }
        self.step_no += 1;
        expand(&self.tree, self.step_no as usize)
    }

    fn match_next(&mut self) -> (bool, Option<String>) {
        let terminal = left_most_empty_terminal(&self.tree);

        if terminal.is_none() {
            return (false, None);
        }

        let terminal = terminal.unwrap();

        let rule = terminal.borrow().rule();

        if rule
            .borrow()
            .matches(&self.tokens.last().unwrap().token_kind)
        {
            terminal.borrow_mut().token = Some(self.tokens.pop().unwrap());
            return (true, None);
        }
        else {
            (
                false,
                Some(format!(
                    "{} != {}",
                    self.tokens.last().unwrap().text,
                    rule.borrow().name()
                )),
            )
        }
    }

    fn is_matching_ready(&self) -> bool {
        return is_matching_ready(&self.tree) == MatchingReady::Ready;
    }

    fn backtrack(&mut self) -> bool {
        if self.step_no < 0 {
            println!("steps all consumed");
            return false;
        }

        loop {
            let backtrack_result = backtrack(&self.tree, &mut self.tokens, self.step_no);
            match backtrack_result {
                Backtrack::NoOp => return false,
                Backtrack::Backtracked { any_alternated, .. } => {
                    self.step_no -= 1;
                    if !any_alternated {
                        return false;
                    }
                }
            }

            if expandable(&self.tree) {
                break;
            }
            if self.step_no < 0 {
                return false;
            }
        }

        return true;
    }
}

#[derive(PartialEq, Eq)]
enum Backtrack {
    NoOp,
    Backtracked {
        any_alternated: bool,
        erase_us: bool,
    },
}

fn backtrack<'a>(
    node: &Rc<RefCell<Node<'a>>>,
    tokens: &mut Vec<Token<'a>>,
    step: isize,
) -> Backtrack {
    let is_my_step = node.borrow().step_no >= (step as usize);
    // println!(
    //     "my_step={}, incoming_step={} my_step={} me={}",
    //     node.borrow().step_no,
    //     step,
    //     is_my_step,
    //     node.borrow().rule_name(),
    // );

    if node.borrow().is_terminal() {
        return if is_my_step {
            if node.borrow().token.is_some() {
                let mut pop = None;
                println!("putting back: {}", node.borrow().token.unwrap().text);
                std::mem::swap(&mut pop, &mut node.borrow_mut().token);
                tokens.push(pop.unwrap());
            }
            Backtrack::Backtracked {
                any_alternated: false,
                erase_us: true,
            }
        }
        else {
            Backtrack::NoOp
        };
    }
    else {
        let mut any = false;
        let mut any_alternated = false;
        let mut erase_us = false;

        {
            let mut node_mut = node.borrow_mut();
            let mut children = vec![];
            std::mem::swap(&mut children, &mut node_mut.children);

            // println!(
            //     "backtracking children of: {}@{}",
            //     node_mut.rule_name(),
            //     node_mut.step_no
            // );
            children.reverse();
            for c in &children {
                let backtrack_result = backtrack(c, tokens, step);
                match backtrack_result {
                    Backtrack::NoOp => {}
                    Backtrack::Backtracked {
                        any_alternated: aa,
                        erase_us: eu,
                    } => {
                        any = true;
                        any_alternated |= aa;
                        erase_us |= eu;
                    }
                }
            }

            children.reverse();
            std::mem::swap(&mut children, &mut node_mut.children);
        }

        if any {
            if erase_us { // || !any_alternated {
                ensure_no_token(&node);
                node.borrow_mut().children.clear();
            }

            if any_alternated {
                Backtrack::Backtracked {
                    any_alternated: true,
                    erase_us: false,
                }
            }
            else if node.borrow().has_next_alt() {
                node.borrow_mut().alternative_no += 1;
                ensure_no_token(&node);
                node.borrow_mut().children.clear();
                Backtrack::Backtracked {
                    any_alternated: true,
                    erase_us: false,
                }
            }
            else {
                Backtrack::Backtracked {
                    any_alternated: false,
                    erase_us: false,
                }
            }
        }
        else {
            Backtrack::NoOp
        }
    }
}

fn ensure_no_token(node: &Rc<RefCell<Node>>) {
    println!("ensuring: {}", &node.borrow());
    if node.borrow().is_terminal() {
        if node.borrow().token.is_some() {
            panic!("still has token: {}", node.borrow());
        }
    }
    else {
        for c in &node.borrow().children {
            ensure_no_token(c);
        }
    }
}

fn expand(node: &Rc<RefCell<Node>>, step_no: usize) -> bool {
    // println!("{}", node.borrow());
    return if node.borrow().is_terminal() {
        false
    }
    else if !node.borrow().children.is_empty() {
        for child in &node.borrow().children {
            if expand(child, step_no) {
                return true;
            }
        }
        false
    }
    else {
        let sub_rules = if node.borrow().is_alternative() {
            if node.borrow().rule().borrow().sub_rules().is_none() {
                // println!("wtf rule: {}", node.borrow().rule().borrow());
                // panic!("wtf?");
                vec![node.borrow().rule()]
            }
            else {
                node.borrow().rule().borrow().sub_rules().unwrap().clone()
            }
        }
        else {
            node.borrow().rules()
        };

        let mut sub_nodes = sub_rules
            .iter()
            .map(|it| Node::child(it, step_no as usize))
            .collect();

        node.borrow_mut().children.append(&mut sub_nodes);

        true
    };
}

fn expandable(node: &Rc<RefCell<Node>>) -> bool {
    if node.borrow().is_terminal() {
        false
    }
    else if !node.borrow().children.is_empty() {
        for child in &node.borrow().children {
            if expandable(child) {
                return true;
            }
        }
        false
    }
    else {
        true
    }
}

#[derive(PartialEq, Eq)]
enum MatchingReady {
    Ready,
    NeedExpand,
    Neutral,
}

fn is_matching_ready(node: &Rc<RefCell<Node>>) -> MatchingReady {
    let node = node.borrow();

    if node.is_terminal() {
        return if node.token.is_none() {
            MatchingReady::Ready
        }
        else {
            MatchingReady::Neutral
        };
    }

    if node.children.is_empty() {
        return MatchingReady::NeedExpand;
    }

    for child in &node.children {
        match is_matching_ready(child) {
            MatchingReady::Ready => {
                return MatchingReady::Ready;
            }
            MatchingReady::NeedExpand => {
                return MatchingReady::NeedExpand;
            }
            MatchingReady::Neutral => {}
        }
    }

    return MatchingReady::Neutral;
}

pub fn parse_inefficiently(
    mut tokens: Vec<Token>,
    rules: Rc<RefCell<Rule>>,
) -> Result<Rc<RefCell<Node>>, String> {
    // crate::lang::parser::parse_tree::sample(&rules);

    tokens.reverse();

    let mut parser = BacktrackingParser {
        tokens,
        tree: Node::root(&rules),
        step_no: 0,
    };

    loop {
        if parser.tokens.last().is_some() {
            println!(
                "\n=========================================================================\n\
            CURRENT TOKEN :::: {}",
                &parser.tokens.last().unwrap()
            );
        }

        let mut expanded = false;
        while !parser.is_matching_ready() {
            expanded = true;
            if !parser.expand() {
                return Err("can not expand".to_string());
            }
        }
        if expanded {
            println!("\nAFTER: {}\n", parser.tree.borrow());
        }

        let (matches, match_reason) = parser.match_next();
        if !matches {
            println!("no match, backtracking => {}", match_reason.unwrap());
            if !parser.backtrack() {
                println!("{}", parser.tree.borrow().rule().borrow());
                return Err("can not backtrack".to_string());
            }
        }
        else {
            println!("MATCH");
        }

        if parser.tokens.is_empty() {
            todo!("check all rules consumed.");
            // return Ok(parser.tree);
        }
    }
}
