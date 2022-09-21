use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::node::display_of;
use crate::lang::parser::node::Node;
use crate::lang::parser::rule::RulePart;
use crate::lang::parser::rules::Rules;

fn is_non_terminal_with_alt(node: &Option<Rc<RefCell<Node<'_>>>>) -> bool {
    node.is_some()
        && node.as_ref().unwrap().borrow().rule_part.is_rule()
        && node.as_ref().unwrap().borrow().has_alt()
}

fn is_token_match(
    node: &Option<Rc<RefCell<Node<'_>>>>,
    word: &Option<Token<'_>>,
) -> bool {
    println!(
        "TRYING TO MATCH: {} <==> {}",
        node.as_ref()
            .map(|it| it.borrow().rule_part.name())
            .or(Some("?".to_string()))
            .unwrap(),
        word.map(|it| it.text).or(Some("?")).unwrap(),
    );

    dbg!(node.is_some());
    dbg!(word.is_some());
    dbg!(node.is_some() && node.as_ref().unwrap().borrow().rule_part.is_token());
    dbg!(
        node.is_some()
            && word.is_some()
            && node.as_ref().unwrap().borrow().rule_part.is_token()
            && node.as_ref().unwrap().borrow().rule_part.get_token_kind()
                == &word.unwrap().token_kind
    );

    node.is_some()
        && word.is_some()
        && node.as_ref().unwrap().borrow().rule_part.is_token()
        && node.as_ref().unwrap().borrow().rule_part.get_token_kind() == &word.unwrap().token_kind
}

fn is_eof(
    node: &Option<Rc<RefCell<Node<'_>>>>,
    word: &Option<Token<'_>>,
) -> bool {
    node.is_none() && word.is_none()
}

fn backtrack<'a>(
    focus: Option<Rc<RefCell<Node<'a>>>>,
    tokens: &mut Vec<Token<'a>>,
) -> Result<Option<Rc<RefCell<Node<'a>>>>, String> {
    //
    println!("NO MATCH");
    let f = focus.unwrap();
    if !f.borrow().rule_part.is_token() && !f.borrow().has_next_alt() && f.borrow().parent.is_none()
    {
        println!("STRAIGHT TO HELL");
        return Err("no more alternative".to_string());
    }
    else {
        println!("LET'S SEE");
        for child in f.borrow().children.iter().rev() {
            if !child.borrow().children.is_empty() {
                panic!("trying to backtrack upper fringe while lower fringe is not backtracked yet")
            }
            else {
                let mut push_back: Option<Token<'a>> = None;
                std::mem::swap(&mut push_back, &mut child.borrow_mut().token);
                println!("putting back? {:?}", push_back);
                if push_back.is_some() {
                    tokens.push(push_back.unwrap());
                }
            }
        }

        if !f.borrow().rule_part.is_token() && f.borrow().has_next_alt() {
            println!("going next");
            f.borrow_mut().next_alt();
            return Ok(Some(f));
        }
        else if f.borrow().parent.is_some() {
            let mut ff = Some(Rc::clone(f.borrow_mut().parent.as_ref().unwrap()));
            return if !ff.as_ref().unwrap().borrow().has_next_alt() {
                backtrack(ff, tokens)
            }
            else {
                ff.as_mut().unwrap().borrow_mut().next_alt();
                Ok(ff)
            };
        }
        else {
            unreachable!("either should have next alt or parent, this is a bug");
        }
    }
}

pub fn parse<'a, T: DoubleEndedIterator<Item = Token<'a>>>(
    rules: &Rules,
    tokens: T,
) -> Result<Rc<RefCell<Node<'a>>>, String> {
    println!("matching against: {}", rules);

    rules.is_valid()?;

    // We're backtracking parser, one more inefficiency is that we need to collect into vector so
    // that we can rewind (is there any rewind-capable rust iterator? if yes let's use that).
    let mut tokens: Vec<Token<'a>> = tokens.rev().collect();
    let mut word = tokens.pop();
    println!("starting with word: {:?}", word);

    let root = {
        let rule_part: RulePart = rules.rules.first().unwrap().into();
        let root: Node<'a> = rule_part.into();
        let root: Rc<RefCell<Node<'a>>> = root.into();
        root
    };

    let mut focus: Option<Rc<RefCell<Node>>> = Some(Rc::clone(&root));
    let mut stack: Vec<Rc<RefCell<Node>>> = vec![];

    let error: String = loop {
        if is_non_terminal_with_alt(&focus) {
            println!("===========================================================");
            println!("BEFORE\n{}", display_of(&root));
            let alt_no = focus.as_mut().unwrap().borrow_mut().alt();
            let mut children: Vec<Rc<RefCell<Node<'a>>>> = vec![];
            for child in &focus
                .as_ref()
                .unwrap()
                .borrow()
                .rule_part
                .get_rule()
                .borrow()
                .alternatives[alt_no]
            {
                let rule_part = child.clone();
                let mut new_node: Node<'a> = rule_part.into();
                new_node.parent = Some(Rc::clone(&focus.as_ref().unwrap()));
                let new_node: Rc<RefCell<Node<'a>>> = new_node.into();
                children.push(new_node);
            }
            for child in children.iter().rev() {
                stack.push(Rc::clone(child));
            }
            focus.as_mut().unwrap().borrow_mut().children = children;
            focus = stack.pop();
            println!("===========================================================");
            println!("AFTER\n{}", display_of(&root));
            println!("===========================================================");
        }
        else if is_token_match(&focus, &word) {
            println!(
                "happy match: {} => {}",
                focus.as_ref().unwrap().borrow().rule_part.name(),
                word.as_ref().unwrap().text
            );
            focus.as_mut().unwrap().borrow_mut().token = word;
            word = tokens.pop();
            focus = stack.pop();
            println!("word is now: {:?}", word);
            if focus.is_some() {
                println!(
                    "focus is now: {}",
                    focus.as_ref().unwrap().borrow().rule_part.name()
                );
            }
            else {
                println!("focus is now: None");
            }
        }
        else if is_eof(&focus, &word) {
            break String::with_capacity(0);
        }
        else {
            println!("NO MATCH");
            if !focus.as_ref().unwrap().borrow().rule_part.is_token()
                && !focus.as_ref().unwrap().borrow().has_next_alt()
                && focus.as_ref().unwrap().borrow().parent.is_none()
            {
                println!("STRAIGHT TO HELL");
                break "no more alternative".to_string();
            }
            else {
                match backtrack(focus, &mut tokens) {
                    Ok(ff) => focus = ff,
                    Err(err) => break err,
                }
            }
        }
    };

    if error.len() == 0 {
        Ok(root)
    }
    else {
        Err(error)
    }
}
