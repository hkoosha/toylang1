use std::cell::RefCell;
use std::rc::Rc;

use log::trace;

use crate::lang::lexer::token::Token;
use crate::lang::lexer::token::TokenKind;
use crate::lang::parser::node::display_of;
use crate::lang::parser::node::Node;
use crate::lang::parser::node::ParseError;
use crate::lang::parser::node::ParseResult;
use crate::lang::parser::rule::RulePart;
use crate::lang::parser::rules::Rules;

fn print_stack(stack: &[Rc<RefCell<Node>>]) {
    trace!(
        "<<<<<<<<<<<<<<<<<<<<<<<<<<<< stack: {}",
        stack
            .iter()
            .map(|it| it.borrow().rule_part().name() + "-" + &it.borrow().num().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn is_non_terminal_with_alt(node: &Option<Rc<RefCell<Node<'_>>>>) -> bool {
    node.is_some()
        && node.as_ref().unwrap().borrow().rule_part().is_rule()
        && node.as_ref().unwrap().borrow().has_alt()
}

fn is_token_match(
    node: &Option<Rc<RefCell<Node<'_>>>>,
    word: &Option<Token<'_>>,
) -> bool {
    trace!(
        "TRYING TO MATCH: {} <==> {}",
        node.as_ref()
            .map(|it| it.borrow().rule_part().name())
            .unwrap_or_else(|| "?".to_string()),
        word.map(|it| it.text).unwrap_or("?"),
    );

    match node {
        Some(node) => {
            let node = node.borrow();
            if node.rule_part().is_token() {
                word.is_some()
                    && *node.rule_part().get_token_kind() == word.as_ref().unwrap().token_kind
            }
            else {
                false
            }
        },
        None => false,
    }
}

fn is_epsilon(node: &Option<Rc<RefCell<Node<'_>>>>) -> bool {
    match node {
        None => false,
        Some(node) => {
            node.borrow().rule_part().is_token()
                && *node.borrow().rule_part().get_token_kind() == TokenKind::Epsilon
        },
    }
}

fn is_eof(
    node: &Option<Rc<RefCell<Node<'_>>>>,
    word: &Option<Token<'_>>,
) -> bool {
    node.is_none() && word.is_none()
}

fn backtrack_push_back<'a>(
    focus: Rc<RefCell<Node<'a>>>,
    tokens: &mut Vec<Token<'a>>,
    stack: &mut Vec<Rc<RefCell<Node>>>,
) {
    if !focus.borrow().children().is_empty() {
        trace!("KILLING CHILDREN OF: {}", focus.borrow().rule_part().name());
        for child in focus.borrow().children().iter().rev() {
            backtrack_push_back(Rc::clone(child), tokens, stack);
        }
        print_stack(stack);
    }
    else if focus.borrow().token().is_some() {
        let push_back = focus.borrow_mut().drain_token();
        trace!(
            "///////////////////////> putting back ::: {} ({})",
            push_back.text,
            focus.borrow().rule_part().name()
        );

        tokens.push(push_back);
    }

    let num = focus.borrow().num();
    stack.retain_mut(|it| it.borrow().num() != num);
}

fn backtrack<'a>(
    focus: Option<Rc<RefCell<Node<'a>>>>,
    tokens: &mut Vec<Token<'a>>,
    stack: &mut Vec<Rc<RefCell<Node>>>,
) -> Result<Option<Rc<RefCell<Node<'a>>>>, String> {
    trace!(
        "NO MATCH backtracking:\n{}\n>>>>>>>>",
        display_of(focus.as_ref().unwrap())
    );
    print_stack(stack);

    let focus = focus.unwrap();
    if !focus.borrow().rule_part().is_token()
        && !focus.borrow().has_next_alt()
        && focus.borrow().parent().is_none()
    {
        trace!("STRAIGHT TO HELL");
        Err(format!(
            "no more alt on: {}",
            focus.borrow().rule_part().name()
        ))
    }
    else {
        trace!("LET'S SEE");
        backtrack_push_back(Rc::clone(&focus), tokens, stack);

        if !focus.borrow().rule_part().is_token() && focus.borrow().has_next_alt() {
            trace!("going next");
            focus.borrow_mut().next_alt();
            Ok(Some(focus))
        }
        else if focus.borrow().parent().is_some() {
            trace!(
                "going parent of: {} aka {}",
                focus.borrow().rule_part().name(),
                focus
                    .borrow()
                    .parent()
                    .as_ref()
                    .map_or("?".to_string(), |it| it.borrow().rule_part().name()),
            );
            let ff = Some(Rc::clone(focus.borrow_mut().parent().as_ref().unwrap()));
            backtrack(ff, tokens, stack)
        }
        else {
            unreachable!("either should have next alt or parent, this is a bug");
        }
    }
}

pub fn parse_with_backtracking<'a, 'b, T: DoubleEndedIterator<Item = Token<'a>>>(
    rules: &Rules,
    tokens: T,
) -> ParseResult<'a> {
    trace!("matching against: {}", rules);

    // We're backtracking parser, one more inefficiency is that we need to collect into vector so
    // that we can rewind (is there any rewind-capable rust iterator? if yes let's use that).
    let mut tokens: Vec<Token<'a>> = tokens.rev().collect();
    let mut word = tokens.pop();
    trace!("starting with word: {:?}", word);

    let mut next_num = 0;

    let root = {
        let rule_part: RulePart = rules.rules().first().unwrap().into();
        let root: Node<'_> = Node::new(rule_part, next_num);
        next_num += 1;
        let root: Rc<RefCell<Node<'_>>> = root.into();
        root
    };

    if let Err(err) = rules.validate() {
        return Err(ParseError::new(&root, format!("invalid rules: {}", err)));
    }

    let mut focus: Option<Rc<RefCell<Node>>> = Some(Rc::clone(&root));
    let mut stack: Vec<Rc<RefCell<Node>>> = vec![];

    let error: String = loop {
        if is_non_terminal_with_alt(&focus) {
            let alt_no = focus.as_mut().unwrap().borrow_mut().alt();
            let mut children: Vec<Rc<RefCell<Node<'a>>>> = vec![];
            for child in &focus
                .as_ref()
                .unwrap()
                .borrow()
                .rule_part()
                .get_rule()
                .borrow()
                .alternatives[alt_no]
            {
                let rule_part = child.clone();
                let new_node: Node<'a> =
                    Node::new_with_parent(rule_part, next_num, focus.as_ref().unwrap());
                next_num += 1;
                let new_node: Rc<RefCell<Node<'a>>> = new_node.into();
                children.push(new_node);
            }
            for child in children.iter().rev() {
                stack.push(Rc::clone(child));
            }
            focus.as_mut().unwrap().borrow_mut().set_children(children);
            focus = stack.pop();
            trace!("===========================================================");
            trace!("AFTER\n{}", display_of(&root));
            print_stack(&stack);
            trace!("===========================================================");
        }
        else if is_epsilon(&focus) {
            trace!("happy epsilon while at: {}", word.as_ref().unwrap().text);
            focus = stack.pop();
            if focus.is_some() {
                trace!(
                    "focus is now: {} vs: {:?}",
                    focus.as_ref().unwrap().borrow().rule_part().name(),
                    word,
                );
            }
            else {
                trace!("focus is now: None, vs: {:?}", word);
            }
        }
        else if is_token_match(&focus, &word) {
            trace!(
                "happy match: {} => {}",
                focus.as_ref().unwrap().borrow().rule_part().name(),
                word.as_ref().unwrap().text,
            );
            focus
                .as_mut()
                .unwrap()
                .borrow_mut()
                .set_token(word.unwrap());
            word = tokens.pop();
            focus = stack.pop();
            match &word {
                None => trace!("word is now: None"),
                Some(word) => trace!("word is now: {}", word.text),
            }
            if focus.is_some() {
                trace!(
                    "focus is now: {} vs: {}",
                    focus.as_ref().unwrap().borrow().rule_part().name(),
                    word.map_or("None", |it| it.text),
                );
            }
            else {
                trace!("focus is now: None, vs: {:?}", word);
            }
        }
        else if is_eof(&focus, &word) {
            trace!("fin!");
            break String::with_capacity(0);
        }
        else {
            if let Some(word) = word {
                tokens.push(word);
            }
            match backtrack(focus, &mut tokens, &mut stack) {
                Ok(ff) => focus = ff,
                Err(err) => break err,
            }
            word = tokens.pop();
        }
    };

    if error.is_empty() {
        Ok(root)
    }
    else {
        Err(ParseError::new(&root, error))
    }
}
