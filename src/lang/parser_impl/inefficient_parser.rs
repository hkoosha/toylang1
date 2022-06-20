use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::parse_tree::{left_most_empty_terminal, Node};
use crate::lang::parser::rule::Rule;

struct BacktrackingParser<'a> {
    tokens: Vec<Token<'a>>,
    tree: Rc<RefCell<Node<'a>>>,
}

impl<'a> BacktrackingParser<'a> {
    fn expand(&mut self) -> bool {
        if is_matching_ready(&self.tree) == MatchingReady::Neutral {
            println!("fuck");
            return false;
        }
        return expand(&self.tree);
    }

    fn match_next(&mut self) -> bool {
        let terminal = left_most_empty_terminal(&self.tree);

        if terminal.is_none() {
            println!("no terminal");
            return false;
        }

        let terminal = terminal.unwrap();
        println!(
            "terminal: {} <=> {}",
            terminal.borrow().rule().borrow().name(),
            &self.tokens.last().unwrap().text
        );
        if terminal
            .borrow()
            .rule()
            .borrow()
            .matches(&self.tokens.last().unwrap().token_kind)
        {
            terminal.borrow_mut().token = Some(self.tokens.pop().unwrap());
            return true;
        }

        false
    }

    fn is_matching_ready(&self) -> bool {
        return is_matching_ready(&self.tree) == MatchingReady::Ready;
    }

    fn backtrack(&mut self) -> bool {
        backtrack(&self.tree, &mut self.tokens) == Backtrack::Backtracked
    }
}

#[derive(PartialEq, Eq)]
enum Backtrack {
    NoOp,
    Backtracked,
}

fn backtrack<'a>(node: &Rc<RefCell<Node<'a>>>, tokens: &mut Vec<Token<'a>>) -> Backtrack {
    if node.borrow().rule().borrow().is_terminal() {
        println!("backtrack terminal");
        if node.borrow().token.is_some() {
            println!(
                "putting back token: {}",
                node.borrow().token.unwrap().token_kind
            );
            let mut pop = None;
            std::mem::swap(&mut pop, &mut node.borrow_mut().token);
            tokens.push(pop.unwrap());
        }
        return Backtrack::NoOp;
    }

    if node.borrow().children.is_empty() {
        return Backtrack::NoOp;
    }

    println!("backtrack children");
    let mut num_to_pop = 0;
    let mut any_child_backtracked = false;
    for c in node.borrow().children.iter().rev() {
        if Backtrack::Backtracked == backtrack(c, tokens) {
            println!("child backtracked");
            any_child_backtracked = true;
        } else {
            num_to_pop += 1;
        }
    }
    for _ in 0..=num_to_pop {
        node.borrow_mut().children.pop();
    }
    if any_child_backtracked {
        return Backtrack::Backtracked;
    }

    if node.borrow().rule().borrow().sub_rules().unwrap().len() <= node.borrow().alternative_no {
        println!("no more alt: {}", node.borrow().rule().borrow().name());
        return Backtrack::NoOp;
    }

    node.borrow_mut().alternative_no += 1;
    return Backtrack::Backtracked;
}

fn expand(node: &Rc<RefCell<Node>>) -> bool {
    if node.borrow().rule().borrow().is_terminal() {
        println!("focus is terminal");
        return false;
    }

    if !node.borrow().children.is_empty() {
        for child in &node.borrow().children {
            if expand(child) {
                return true;
            }
        }
        return false;
    }

    let node_borrow = node.borrow();
    let rule = node_borrow.rule();
    let rule_borrow = rule.borrow();
    // let sub_rule: &Rc<RefCell<Rule>> = rule_borrow.sub_rules().unwrap().get(alt).unwrap();
    let mut sub_nodes = rule_borrow
        // .borrow()
        .sub_rules()
        .unwrap()
        .iter()
        .map(|it| Node::child(it))
        .collect();

    println!("rule_borrow: {}", rule_borrow);

    drop(rule_borrow);
    drop(node_borrow);

    node.borrow_mut().children.append(&mut sub_nodes);

    return true;
}

#[derive(PartialEq, Eq)]
enum MatchingReady {
    Ready,
    NeedExpand,
    Neutral,
}

fn is_matching_ready(node: &Rc<RefCell<Node>>) -> MatchingReady {
    let node = node.borrow();

    if node.rule().borrow().is_terminal() {
        return if node.token.is_none() {
            MatchingReady::Ready
        } else {
            MatchingReady::Neutral
        };
    }

    if node.children.is_empty() {
        println!("================================ NO CHILDREN, need Expanding");
        return MatchingReady::NeedExpand;
    }

    println!("with children: {} ////////////", node);
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

    println!("=======================================================================> Neutral");
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
    };

    loop {
        if parser.tokens.last().is_some() {
            println!("CURRENT :::: {}", &parser.tokens.last().unwrap());
        }
        println!("EXPANDING?");
        while !parser.is_matching_ready() {
            println!("-------------------------------> EXPANDING");
            if !parser.expand() {
                return Err("can not expand".to_string());
            } else {
                println!("expanded");
            }
        }
        println!("EXPANDING!");

        println!(
            "matches? {} {}",
            parser.tree.borrow(),
            &parser.tokens.last().unwrap()
        );
        if !parser.match_next() {
            println!("no match");
            if !parser.backtrack() {
                return Err("can not backtrack".to_string());
            }
        } else {
            println!("MATCH");
        }

        if parser.tokens.is_empty() {
            todo!("check all rules consumed.");
            // return Ok(parser.tree);
        }
    }
}
