use std::cell::RefCell;
use std::rc::Rc;

use log::info;

use crate::lang::lexer::token::Token;
use crate::lang::parser::grammar::rules;
use crate::lang::parser::rule::RuleNode;

pub fn parse<'a, T>(tokens: T, rules: Rc<RefCell<RuleNode>>)
    where
        T: IntoIterator<Item=Token<'a>>,
{
    //

    for t in tokens.into_iter() {}
}
