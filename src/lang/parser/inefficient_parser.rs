use log::info;

use crate::lang::parser::grammar::rules;
use crate::lang::token::Token;

pub fn parse<'a, T>(tokens: T)
    where T: IntoIterator<Item=Token<'a>> {
    //

    let r = rules();

    info!("\n==============================================\n{}\n========================================\n", *r.borrow());

    for t in tokens.into_iter() {
        info!("t: {}", t);
    }
}