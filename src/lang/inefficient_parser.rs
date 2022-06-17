use crate::lang::lexer::Token;
use crate::lang::lexer::TokenKind;

#[derive(Clone)]
pub enum Rule {
    Terminal(TokenKind),
    Expandable(Vec<Rule>),
}

pub fn parse<'a, R, T>(rules: R, tokens: T)
    where R: IntoIterator<Item=Rule>,
          T: IntoIterator<Item=Token<'a>> {
    //

    for t in tokens.into_iter() {
        println!("t: {}", t);
    }
}