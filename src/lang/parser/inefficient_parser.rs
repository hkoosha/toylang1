use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::rule::RuleNode;

pub fn parse<'a, T>(tokens: T, rules: Rc<RefCell<RuleNode>>)
where
    T: IntoIterator<Item = Token<'a>>,
{
    //

    let mut i: usize = 0;
    let tokens: Vec<Token<'a>> = tokens.into_iter().collect();
    let mut word = Some(&tokens[i]);

    let root = Rc::clone(&rules);
    let mut focus = Some(&root);

    let mut stack: Vec<Option<RuleNode>> = vec![];
    stack.push(None);

    loop {
        if focus.is_some() && focus.unwrap().borrow().is_non_terminal() {
        }
        else if focus.is_some()
            && word.is_some()
            && focus.unwrap().borrow().matches(&word.unwrap().token_kind)
        {
        }
        else if word.is_none() && focus.is_none() {
            return;
        }
        else {
            // backtrack;
        }
    }
}
