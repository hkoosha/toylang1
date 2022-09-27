use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::parser::node::Node;
use crate::lang::parser::node::ParseError;
use crate::lang::parser::rules::Rules;

pub fn parse<'a, T: DoubleEndedIterator<Item = Token<'a>>>(
    rules: &Rules,
    tokens: T,
) -> Result<Rc<RefCell<Node<'a>>>, ParseError<'a>> {
    todo!()
}
