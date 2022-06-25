use std::cell::RefCell;
use std::rc::Rc;

use log::trace;

use crate::lang::lexer::token::Token;
use crate::lang::parser::parse_tree::{ensure_parent_sane, root_of, Node};
use crate::lang::parser::rule::Rule;

struct BacktrackingParser<'a> {
    tokens: Vec<Token<'a>>,
    focus: Rc<RefCell<Node<'a>>>,
    stack: Vec<Rc<RefCell<Node<'a>>>>,
    step_no: usize,
}

impl<'a> BacktrackingParser<'a> {
    fn expand(&mut self) {
        if self.focus.borrow().is_terminal() {
            panic!("expand on terminal: {}", self.focus.borrow());
        }
        if !self.focus.borrow().children.is_empty() {
            panic!("already has children: {}", self.focus.borrow());
        }

        let sub_rules = if self.focus.borrow().is_expandable() {
            self.focus.borrow().expandable_rules()
        }
        else {
            // if self.focus.borrow().rule().borrow().sub_rules().is_none() {
            vec![self.focus.borrow().alt_current_rule()]
        };
        // else {
        //     trace!(
        //         "=======================> EXP: {} => ",
        //         self.focus.borrow().rule().borrow().name(),
        //     );
        //     self.focus
        //         .borrow()
        //         .rule()
        //         .borrow()
        //         .sub_rules()
        //         .unwrap()
        //         .clone()
        // };

        let mut sub_nodes: Vec<Rc<RefCell<Node>>> = sub_rules
            .iter()
            .map(|it| Node::child(it, &self.focus, self.step_no))
            .collect();

        self.step_no += 1;

        self.print_stack("before sub nodes");
        (&sub_nodes)
            .into_iter()
            .rev()
            .map(|it| Rc::clone(it))
            .for_each(|it| self.stack.push(it));
        self.print_stack("after sub nodes");

        assert!(!sub_nodes.is_empty());

        self.focus.borrow_mut().children.append(&mut sub_nodes);
        self.print_stack("before focus");
        self.focus.borrow_mut().is_focus = false;
        self.focus = self.stack.pop().unwrap();
        self.focus.borrow_mut().is_focus = true;
        self.print_stack("after focus");
    }

    fn match_next(&mut self) -> bool {
        trace!(
            "matches? {} => {}",
            self.focus.borrow().rule_name(),
            self.tokens.last().unwrap().text
        );
        let rule = self.focus.borrow().terminal_rule();
        if rule
            .borrow()
            .matches(&self.tokens.last().unwrap().token_kind)
        {
            self.focus.borrow_mut().token = Some(self.tokens.pop().unwrap());
            self.print_stack("before match");
            self.focus.borrow_mut().is_focus = false;

            if !self.stack.is_empty() {
                self.focus = self.stack.pop().unwrap();
                self.focus.borrow_mut().is_focus = true;
            }
            self.print_stack("after match");
            true
        }
        else {
            false
        }
    }

    fn backtrack(&mut self) -> Backtrack {
        assert!(self.focus.borrow().children.is_empty(), "has no children");

        trace!("BACK: {}", self.focus.borrow().rule_name());

        if self.focus.borrow().parent.as_ref().is_none() {
            return Backtrack::Fin;
        }

        let parent = Rc::clone(self.focus.borrow().parent.as_ref().expect("no parent!"));
        self.focus = parent;
        self.focus.borrow_mut().is_focus = true;

        let node = Rc::clone(&self.focus);

        disconnect(self, &node);

        return if self.focus.borrow().has_next_alt() {
            self.focus.borrow_mut().alternative_no += 1;
            Backtrack::Yes
        }
        else {
            Backtrack::No
        };
    }

    fn print_stack(&self, tag: &'static str) {
        let mut values = String::new();
        (&self.stack)
            .iter()
            .map(|it| it.borrow().rule_name())
            .for_each(|it| {
                values.push(' ');
                values.push_str(&it);
            });
        trace!("===> stack {}: {}", tag, values);
    }
}

fn disconnect<'a>(parser: &mut BacktrackingParser<'a>, node: &Rc<RefCell<Node<'a>>>) {
    trace!("dis: {}", node.borrow());

    node.borrow_mut().children.reverse();

    for child in &node.borrow().children {
        disconnect(parser, child);
    }

    trace!("I am {}", node.borrow());
    if !node.borrow().is_focus && node.borrow().is_terminal() && node.borrow().token.is_some() {
        let mut pop = None;
        trace!("putting back: {}", node.borrow().token.unwrap().text);
        std::mem::swap(&mut pop, &mut node.borrow_mut().token);
        parser.tokens.push(pop.unwrap());
    }
    else if !node.borrow().is_focus && node.borrow().children.is_empty() {
        trace!(
            "I am popper: {} - {}",
            node.borrow(),
            node.borrow().is_focus
        );
        trace!(
            "{} / {}",
            node.borrow().parent.as_ref().unwrap().borrow().is_focus,
            node.borrow().is_focus
        );
        parser.print_stack("popping disconnect");
        parser.stack.pop().expect("stack was empty!");
    }

    node.borrow_mut().children.clear();
}

enum Backtrack {
    Yes,
    No,
    Fin,
}

pub fn parse_inefficiently(
    mut tokens: Vec<Token>,
    rules: Rc<RefCell<Rule>>,
) -> Result<Rc<RefCell<Node>>, String> {
    // crate::lang::parser::parse_tree::sample(&rules);

    tokens.reverse();

    let root = Node::root(&rules);
    let mut parser = BacktrackingParser {
        tokens,
        focus: root,
        stack: vec![],
        step_no: 0,
    };

    loop {
        ensure_parent_sane(&root_of(&parser.focus));

        if parser.tokens.last().is_some() {
            trace!(
                "\n=========================================================================\n\
            CURRENT TOKEN :::: {}",
                &parser.tokens.last().unwrap()
            );
        }

        while !parser.focus.borrow().is_terminal() {
            parser.expand();
            ensure_parent_sane(&root_of(&parser.focus));
            trace!("expanded: {}", root_of(&parser.focus).borrow());
        }

        if parser.stack.is_empty() && parser.tokens.is_empty() {
            return Ok(root_of(&parser.focus));
        }

        if parser.match_next() {
            trace!("MATCH");
        }
        else {
            loop {
                match parser.backtrack() {
                    Backtrack::Yes => {
                        trace!("backtracked: {}", root_of(&parser.focus).borrow());
                        trace!("backtracked focus: {}", parser.focus.borrow());
                        parser.print_stack("backtracked");
                        ensure_parent_sane(&root_of(&parser.focus));
                        break;
                    }
                    Backtrack::No => {
                        trace!("backtracked: {}", root_of(&parser.focus).borrow());
                    }
                    Backtrack::Fin => return Err("can not backtrack".to_string()),
                }
                trace!("backtracked: {}", root_of(&parser.focus).borrow());
                ensure_parent_sane(&root_of(&parser.focus));
            }
        }
    }
}
