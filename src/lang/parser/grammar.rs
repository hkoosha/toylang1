use std::cell::RefCell;
use std::rc::Rc;

use crate::lang::lexer::token::TokenKind;
use crate::lang::parser::rule::{Rule, ToRule};

#[allow(dead_code)]
const GRAMMAR: &str = "

S               -> fn_call | fn_declaration
fn_call         -> IDT ( args ) ;
args            -> args0 | arg
args0           -> arg , args
arg             -> TXT | INT | IDT
fn_declaration  -> fn IDT ( params ) { statements }
params          -> params0 | param
params0         -> param , params
param           -> IDT IDT
statements      -> statements0 | statement
statements0     -> statement statements
statement       -> declaration | assignment | fn_call | return
declaration     -> IDT IDT ;
assignment      -> IDT = expressions ;
expressions     -> expression0 | expression1 | terms
expression0     -> terms + expressions
expression1     -> terms - expressions
terms           -> term0 | term1 | factor
term0           -> factor * terms
term1           -> factor / terms
factor          -> factor0 | INT | IDT
factor0         -> ( expressions )
return          -> RET expressions;

";

pub fn toylang_v0_rules() -> Rc<RefCell<Rule>> {
    let num0 = Rc::new(RefCell::new(1_usize));
    let num1 = Rc::clone(&num0);
    let num2 = Rc::clone(&num0);

    let get_num = move || {
        let curr = *num0.borrow();
        *num0.borrow_mut() = curr + 1;
        curr
    };

    let exp = move |name: &'static str, rules: Vec<&Rc<RefCell<Rule>>>| {
        let e_num = *num1.borrow();
        *num1.borrow_mut() = e_num + 1;
        Rc::new(RefCell::new(Rule::Expandable {
            name: name.to_string(),
            num: e_num,
            sub_rules: rules.iter().map(|it| Rc::clone(it)).collect(),
        }))
    };
    let push = |into: &Rc<RefCell<Rule>>, item: &Rc<RefCell<Rule>>| match &mut *Rc::clone(into)
        .borrow_mut()
    {
        Rule::Expandable {
            sub_rules: rules, ..
        } => {
            rules.push(Rc::clone(item));
        }
        Rule::Alternative {
            sub_rules: rules, ..
        } => {
            rules.push(Rc::clone(item));
        }
        _ => panic!(),
    };

    let push_all = |into: &Rc<RefCell<Rule>>, item: Vec<&Rc<RefCell<Rule>>>| {
        for i in item {
            push(into, i);
        }
    };

    let alt = move |name: &'static str, rules: Vec<&Rc<RefCell<Rule>>>| {
        let alt_num = *num2.borrow();
        *num2.borrow_mut() = alt_num + 1;
        Rc::new(RefCell::new(Rule::Alternative {
            name: name.to_string(),
            num: alt_num,
            sub_rules: rules.iter().map(|it| Rc::clone(it)).collect(),
        }))
    };

    let rbc: Rc<RefCell<Rule>> = TokenKind::Rbc.to_rule(get_num());
    let lbc = TokenKind::Lbc.to_rule(get_num());
    let lpr = TokenKind::Lpr.to_rule(get_num());
    let rpr = TokenKind::Rpr.to_rule(get_num());
    let fun = TokenKind::Fun.to_rule(get_num());
    let semi = TokenKind::Smi.to_rule(get_num());
    let comma = TokenKind::Com.to_rule(get_num());
    let equ = TokenKind::Equ.to_rule(get_num());
    let identifier = TokenKind::Idt.to_rule(get_num());
    let int = TokenKind::Int.to_rule(get_num());
    let mul = TokenKind::Mul.to_rule(get_num());
    let div = TokenKind::Sls.to_rule(get_num());
    let plus = TokenKind::Pls.to_rule(get_num());
    let minus = TokenKind::Min.to_rule(get_num());
    let text = TokenKind::Str.to_rule(get_num());
    let ret_token = TokenKind::Ret.to_rule(get_num());

    let param = exp("param", vec![&identifier, &identifier]);
    let params0 = exp("params0", vec![&param, &comma]);
    let params = alt("params", vec![&params0, &param]);
    push(&params0, &params);

    // -----------------------------

    let arg = alt("arg", vec![&text, &identifier, &int]);
    let args = alt("args", vec![]);
    let args0 = exp("params0", vec![&arg, &comma, &args]);
    push_all(&args, vec![&args0, &arg]);

    let fn_call = exp("fn_call", vec![&identifier, &lpr, &args, &rpr, &semi]);

    // -----------------------------

    let term0 = exp("term0", vec![]);
    let term1 = exp("term1", vec![]);
    let terms = alt("terms", vec![&term0, &term1]);

    let factor0 = exp("factor0", vec![]);
    let factor = alt("factor", vec![&factor0, &int, &identifier]);

    let ret = exp("return", vec![&ret_token]);

    let expression0 = exp("expression0", vec![]);
    let expression1 = exp("expression1", vec![]);
    let expressions = alt("expressions", vec![&expression0, &expression1, &terms]);

    push_all(&expression0, vec![&terms, &plus, &expressions]);
    push_all(&expression1, vec![&terms, &minus, &expressions]);
    push_all(&ret, vec![&expressions, &semi]);

    push_all(&factor0, vec![&lpr, &expressions, &rpr]);
    push_all(&term0, vec![&factor, &mul, &terms]);
    push_all(&term1, vec![&factor, &div, &terms]);
    push_all(&terms, vec![&factor]);

    // -----------------------------

    let declaration = exp("declaration", vec![&identifier, &identifier, &semi]);
    let assignment = exp("assignment", vec![&identifier, &equ, &expressions, &semi]);

    // -----------------------------

    let statement = alt("statement", vec![&fn_call, &declaration, &assignment, &ret]);

    let statements0 = exp("statements0", vec![&statement]);
    let statements = alt("statements", vec![&statements0, &statement]);
    push(&statements0, &statements);

    // -----------------------------

    let fn_declaration = exp(
        "fn_declaration",
        vec![
            &fun,
            &identifier,
            &lpr,
            &params,
            &rpr,
            &lbc,
            &statements,
            &rbc,
        ],
    );

    alt("S", vec![&fn_call, &fn_declaration])
}
