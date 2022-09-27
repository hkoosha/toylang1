use std::collections::BTreeMap;
use std::collections::BTreeSet;

use toylang1::lang::lexer::token::TokenKind;
use toylang1::lang::lexer::v0::Lexer;
use toylang1::lang::parser::node::display_of;
use toylang1::lang::parser::rules::Rules;
use toylang1::lang::parser_impl::backtracking_parser::parse;

const SAMPLE_CORRECT_PROGRAM: &str = "\
    fn my_thing42(int j) {
        x1 = 1 * 30;
        x2 = x3 / 10;
        int y;
        y = x4 + 2;
        int z;
        print(\"foo\\\"bar \\some thing\");
        z = x5 * y;
        print(z);
        int x0;
        return x0 + 0;
    }";

const SAMPLE_INCORRECT_PROGRAM: &str = "\
    fn my_thing42(int j) {
    ";

const GRAMMAR: &str = "

S               -> fn_call | fn_declaration
fn_call         -> ID ( args ) ;
args            -> arg , args | arg |
arg             -> STRING | INT | ID
fn_declaration  -> FN ID ( params ) { statements }
params          -> param , params | param |
param           -> ID ID
statements      -> statement statements | statement |
statement       -> ID ID ; | ID = expressions ; | fn_call | ret
expressions     -> terms + expressions | terms - expressions | terms
terms           -> factor * terms | factor / terms | factor
factor          -> ( expressions ) | INT | ID
ret             -> RETURN expressions ;

";

#[allow(clippy::needless_collect)]
fn correct_program(rules: &Rules) -> Result<(), String> {
    let lexer: Lexer = SAMPLE_CORRECT_PROGRAM.into();
    for token in lexer {
        token?;
    }

    let lexer: Lexer = SAMPLE_CORRECT_PROGRAM.into();
    // Parsed successfully above, ok to unwrap.
    let tokens: Vec<_> = lexer.into_iter().map(|it| it.unwrap()).collect();

    let parsed = parse(rules, tokens.into_iter());

    match parsed {
        Ok(parse_tree) => {
            let display = display_of(&parse_tree);
            println!("parsed successfully:\n{}", display);
        },
        Err(parse_error) => {
            return Err(format!("unexpected error: {}", parse_error));
        },
    }

    Ok(())
}

#[allow(clippy::needless_collect)]
fn incorrect_program(rules: &Rules) -> Result<(), String> {
    let lexer: Lexer = SAMPLE_INCORRECT_PROGRAM.into();
    for token in lexer {
        token?;
    }

    let lexer: Lexer = SAMPLE_INCORRECT_PROGRAM.into();
    // Parsed successfully above, ok to unwrap.
    let tokens: Vec<_> = lexer.into_iter().map(|it| it.unwrap()).collect();

    let parsed = parse(rules, tokens.into_iter());

    match parsed {
        Ok(parse_tree) => {
            panic!(
                "expecting error, got parse tree: {}",
                &display_of(&parse_tree)[0..32]
            );
        },
        Err(parse_error) => {
            println!(
                "parsed unsuccessfully as expected, error={}, partial tree:\n{}",
                parse_error.error(),
                display_of(parse_error.partial_tree())
            );
        },
    }

    Ok(())
}

#[allow(clippy::needless_collect)]
fn main() -> Result<(), String> {
    let mut rules: Rules = GRAMMAR.try_into()?;
    rules.eliminate_left_recursions();
    rules.validate()?;

    println!("\n\n===================================================\n\n");
    correct_program(&rules)?;

    println!("\n\n===================================================\n\n");
    incorrect_program(&rules)?;

    println!("\n\n===================================================\n\n");

    println!("RULES: {}\n", rules);

    rules
        .first_set()
        .into_iter()
        .filter(|it| TokenKind::from_name(&it.0).is_err())
        .map(|it| (it.0, it.1.into_iter().collect::<BTreeSet<_>>()))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .for_each(|it| println!("first of {} => {:?}", it.0, it.1));

    println!("\n\n===================================================\n\n");

    rules
        .follow_set()
        .into_iter()
        .map(|it| (it.0, it.1.into_iter().collect::<BTreeSet<_>>()))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .for_each(|it| println!("follow of {} => {:?}", it.0, it.1));

    println!("\n\n===================================================\n\n");

    rules
        .start_set()
        .into_iter()
        .map(|it| (it.0, it.1.into_iter().collect::<BTreeSet<_>>()))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .for_each(|it| println!("follow of {} => {:?}", it.0, it.1));

    println!("\n\n===================================================\n\n");

    rules.make_ready_for_recursive_decent(128)?;

    rules.is_backtrack_free()?;

    println!("backtrack-free: {}", rules);

    println!("\n\n");

    Ok(())
}
