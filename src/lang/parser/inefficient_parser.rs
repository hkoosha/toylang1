use std::cell::RefCell;
use std::rc::Rc;

use log::info;

use crate::lang::parser::Rule;
use crate::lang::token::{Token, TokenKind};

#[allow(dead_code)]
const GRAMMAR: &str = "

S -> fn_declaration | fn_call

fn_call -> identifier '(' args ')' ';'

fn_declaration -> 'fn' '(' args ')' '{' func_body '}'

func_body -> identifier identifier ';'

args -> arg ',' args | arg
arg -> identifier identifier

identifier -> [a-zA-Z][a-zA-Z0-9]*

";

fn rules() -> Rc<RefCell<Rule>> {
    let mut n = 0;
    let mut expandable = move |name: String, rules: Vec<Rc<RefCell<Rule>>>| {
        n += 1;
        Rc::new(RefCell::new(Rule::Expandable {
            name,
            num: n,
            rules,
        }))
    };

    let comma = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Com)));
    let identifier = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Idt)));

    let arg = expandable("arg".to_string(), vec![
        identifier.clone(),
        identifier,
    ]);

    let args = expandable("args".to_string(), vec![
        arg,
        comma,
    ]);
    let args0 = Rc::clone(&args);
    let args1 = Rc::clone(&args);

    match &mut *args0.borrow_mut() {
        Rule::Terminal(_) => panic!(),
        Rule::Expandable { rules, .. } => {
            rules.push(args1);
        }
    }

    args0
}


pub fn parse<'a, T>(tokens: T)
    where T: IntoIterator<Item=Token<'a>> {
    //

    let r = rules();

    println!("====================================");
    println!("{}", r.borrow());
    println!("====================================");

    for t in tokens.into_iter() {
        info!("t: {}", t);
    }
}