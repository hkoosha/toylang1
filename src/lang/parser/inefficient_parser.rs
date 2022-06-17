use std::cell::RefCell;
use std::rc::Rc;

use log::info;

use crate::lang::parser::Rule;
use crate::lang::token::{Token, TokenKind};

#[allow(dead_code)]
const GRAMMAR: &str = "

S -> fn_declaration | fn_call

fn_call -> identifier '(' args ')' ';'

fn_declaration -> 'fn' '(' args ')' '{' fn_body '}'

fn_body -> identifier identifier ';'

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

    let lbc = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Rbc)));
    let rbc = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Lbc)));
    let lpr = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Lpr)));
    let rpr = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Rpr)));
    let fn_ = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Fun)));
    let semi = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Smi)));
    let comma = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Com)));
    let identifier = Rc::new(RefCell::new(Rule::Terminal(TokenKind::Idt)));

    let arg = expandable("arg".to_string(), vec![
        Rc::clone(&identifier),
        Rc::clone(&identifier),
    ]);

    let args = expandable("args".to_string(), vec![
        arg,
        comma,
    ]);
    match &mut *Rc::clone(&args).borrow_mut() {
        Rule::Terminal(_) => panic!(),
        Rule::Expandable { rules, .. } => {
            rules.push(Rc::clone(&args));
        }
    }

    let fn_body = expandable("fn_body".to_string(), vec![
        Rc::clone(&identifier),
        Rc::clone(&identifier),
        Rc::clone(&semi),
    ]);

    let fn_declaration = expandable("fn_declaration".to_string(), vec![
        fn_,
        Rc::clone(&lpr),
        Rc::clone(&args),
        Rc::clone(&rpr),
        lbc,
        fn_body,
        rbc,
    ]);

    // fn_call -> identifier '(' args ')' ';'

    let fn_call = expandable("fn_call".to_string(), vec![
        Rc::clone(&identifier),
        Rc::clone(&lpr),
        Rc::clone(&args),
        Rc::clone(&rpr),
        Rc::clone(&semi),
    ]);

    let s = expandable("S".to_string(), vec![
        fn_call,
        fn_declaration,
    ]);

    s
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