use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::parse_tree::Node;
use crate::lang::parser::rule::Rule;

struct BacktrackingParser<'a> {
    tokens: Vec<Token<'a>>,
    tree: Node<'a>,
}

impl<'a> BacktrackingParser<'a> {
    /*    fn parse(&mut self, tree: &mut Node<'a>) -> Result<(), String> {
        if self.focus.as_ref().is_some() {
            println!(
                "focus: {}  <==> tk: {}, t: {}",
                &self.focus.as_ref().unwrap().borrow().name(),
                &self.tokens[self.tokens_i].token_kind,
                &self.tokens[self.tokens_i].text,
            );
        }

        return if self.focus.is_none() {
            Err("no focus".to_string())
        }
        else if self.focus.as_ref().unwrap().borrow().is_non_terminal() {
            let focus = Rc::clone(&*self.focus.as_ref().unwrap());
            let focus_borrow = &*focus.borrow();
            match focus_borrow {
                Rule::Terminal(_) => unreachable!(),
                Rule::Expandable { rules, .. } => {
                    for rule in rules.iter().rev() {
                        println!("expandable sub-rule: {}", rule.borrow().name());
                        let child = Node::child(rule);
                        tree.children.push(child);
                    }
                    self.focus = Some(Rc::clone(&tree.children.last().unwrap().rule));
                    self.parse(tree)
                }
                Rule::Alternative { rules, .. } => {
                    for alternative in rules {
                        println!("alternative: {}", alternative.borrow().name());
                        self.focus = Some(Rc::clone(&alternative));
                    }
                    self.parse(tree)
                }
            }
        }
        else if self
            .focus
            .as_ref()
            .unwrap()
            .borrow()
            .matches(&self.tokens[self.tokens_i].token_kind)
        {
            println!("match");
            self.tokens_i += 1;
            // self.focus = self.stack.pop();
            todo!("set self focus");
            Ok(())
        }
        else {
            Err("no match".to_string())
        };
    }*/

    fn expand(&mut self) -> bool {
        return expand(&mut self.tree);
    }

    fn match_next(&mut self) -> bool {
        let terminal = self.tree.left_most_empty_terminal();

        if terminal.is_none() {
            return false;
        }

        let terminal = terminal.unwrap();
        if terminal
            .rule
            .borrow()
            .matches(&self.tokens.last().unwrap().token_kind)
        {
            terminal.token = Some(self.tokens.pop().unwrap());
            return true;
        }

        false
    }

    fn is_matching_ready(&self) -> bool {
        return is_matching_ready(&self.tree);
    }

    fn backtrack(&mut self) -> bool {
        backtrack(&mut self.tree, &mut self.tokens);
        false
    }
}

enum Backtrack {
    NoOp,
    Backtracked,
}

fn backtrack<'a>(node: &mut Node<'a>, tokens: &mut Vec<Token<'a>>) -> Backtrack {
    if node.token.is_some() {
        return Backtrack::NoOp;
    }

    for c in &mut node.children {
        match backtrack(c, tokens) {
            Backtrack::NoOp => {}
            Backtrack::Backtracked => {
                return Backtrack::Backtracked;
            }
        }
    }

    if node.rule.borrow().sub_rules().unwrap().len() <= node.alternative_no {
    }

    return Backtrack::NoOp;
}

fn expand(node: &mut Node) -> bool {
    if node.rule.borrow().is_terminal() {
        return false;
    }

    if !node.children.is_empty() {
        for child in &mut node.children {
            if expand(child) {
                return true;
            }
        }
        return false;
    }

    for sub_rule in node.rule.borrow().sub_rules().unwrap() {
        node.children.push(Node::child(sub_rule));
    }

    return true;
}

fn is_matching_ready(node: &Node) -> bool {
    if node.rule.borrow().is_terminal() {
        return node.token.is_none();
    }

    if !node.children.is_empty() {
        for child in &node.children {
            if is_matching_ready(child) {
                return true;
            }
        }
    }

    return false;
}

pub fn parse_inefficiently(tokens: Vec<Token>, rules: Rc<RefCell<Rule>>) -> Result<Node, String> {
    // crate::lang::parser::parse_tree::sample(&rules);

    let mut parser = BacktrackingParser {
        tokens,
        tree: Node::root(&rules),
    };

    loop {
        while !parser.is_matching_ready() {
            if !parser.expand() {
                return Err("can not expand".to_string());
            }
            else {
                println!("expanded");
            }
        }

        println!("matches? {}", parser.tree);
        if !parser.match_next() {
            println!("no match");
            if !parser.backtrack() {
                return Err("can not backtrack".to_string());
            }
        }

        if parser.tokens.is_empty() {
            return Ok(parser.tree);
        }
    }
}
