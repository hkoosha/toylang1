use std::cell::RefCell;
use std::rc::Rc;

use log::info;

use crate::lang::parser::rule::RuleNode;
use crate::lang::parser::rule::ToRule;
use crate::lang::token::{Token, TokenKind};

#[allow(dead_code)]
const GRAMMAR: &str = "

S -> fn_declaration | fn_call

fn_call -> identifier '(' args ')' ';'

fn_declaration -> 'fn' '(' args ')' '{' fn_body '}'

fn_body -> identifier identifier ';'

args -> arg ',' args | arg
arg -> identifier identifier

identifier -> IDT

";

fn rules() -> Rc<RefCell<RuleNode>> {
    let mut e_num = 0;
    let mut expandable = move |name: &'static str, rules: Vec<Rc<RefCell<RuleNode>>>| {
        e_num += 1;
        Rc::new(RefCell::new(RuleNode::Expandable {
            name: name.to_string(),
            num: e_num,
            rules,
        }))
    };

    let mut a_num = 0;
    let mut alternative = move |name: &'static str, rules: Vec<Rc<RefCell<RuleNode>>>| {
        a_num += 1;
        Rc::new(RefCell::new(RuleNode::Alternative {
            name: name.to_string(),
            num: a_num,
            rules,
        }))
    };

    let lbc: Rc<RefCell<RuleNode>> = TokenKind::Rbc.to_rule();
    let rbc = TokenKind::Lbc.to_rule();
    let lpr = TokenKind::Lpr.to_rule();
    let rpr = TokenKind::Rpr.to_rule();
    let fn_ = TokenKind::Fun.to_rule();
    let semi = TokenKind::Smi.to_rule();
    let comma = TokenKind::Com.to_rule();
    let identifier = TokenKind::Idt.to_rule();

    let arg = expandable("arg", vec![
        Rc::clone(&identifier),
        Rc::clone(&identifier),
    ]);

    let args0 = expandable("args0", vec![
        Rc::clone(&arg),
        comma,
    ]);
    let args1 = expandable("args1", vec![
        Rc::clone(&arg),
    ]);
    let args = alternative("args", vec![
        Rc::clone(&args0),
        Rc::clone(&args1),
    ]);
    match &mut *Rc::clone(&args0).borrow_mut() {
        RuleNode::Expandable { rules, .. } => {
            rules.push(Rc::clone(&args));
        }
        _ => panic!(),
    }

    let statements = expandable("statements", vec![
        Rc::clone(&identifier),
        Rc::clone(&identifier),
        Rc::clone(&semi),
    ]);

    let fn_declaration = expandable("fn_declaration", vec![
        fn_,
        Rc::clone(&lpr),
        Rc::clone(&args),
        Rc::clone(&rpr),
        lbc,
        statements,
        rbc,
    ]);

    let fn_call = expandable("fn_call", vec![
        Rc::clone(&identifier),
        Rc::clone(&lpr),
        Rc::clone(&args),
        Rc::clone(&rpr),
        Rc::clone(&semi),
    ]);

    let s = alternative("S", vec![
        fn_call,
        fn_declaration,
    ]);

    return s;
}


pub fn parse<'a, T>(tokens: T)
    where T: IntoIterator<Item=Token<'a>> {
    //

    let r = rules();

    println!("====================================");
    println!("{}", *r.borrow());
    println!("====================================");

    for t in tokens.into_iter() {
        info!("t: {}", t);
    }
}