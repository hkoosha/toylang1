use std::collections::BTreeMap;
use std::collections::BTreeSet;

use log::trace;
use pretty_env_logger::formatted_builder;
use toylang1::lang::lexer::token::TokenKind;
use toylang1::lang::lexer::v0::Lexer;
use toylang1::lang::parser::node::display_of;
use toylang1::lang::parser::rules::Rules;
use toylang1::lang::parser_impl::backtracking_parser::parse_with_backtracking;
use toylang1::lang::parser_impl::recursive_descent_parser::recursive_descent_parse;

#[allow(dead_code)]
const SAMPLE_CORRECT_PROGRAM_0: &str = "\
    fn my_thing42(int j, string q) {
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

#[allow(dead_code)]
const SAMPLE_CORRECT_PROGRAM_1: &str = "\
    fn my_thing42() {
        print(\"hell\");
    }";

#[allow(dead_code)]
const SAMPLE_INCORRECT_PROGRAM: &str = "\
    fn my_thing42(int j) {
    ";

#[allow(dead_code)]
const SAMPLE_UNPARSABLE_PROGRAM: &str = "\
    fn my_thing42(int j) {
        123abc = 1 * 2;
    }
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

#[allow(dead_code)]
fn yes() -> bool {
    true
}

#[allow(dead_code)]
fn en_log() {
    let mut builder = formatted_builder();
    builder.parse_filters("trace");
    builder.try_init().unwrap();
    trace!("log enabled");
}

fn first_follow_start(rules: &Rules) {
    println!("\n\n===================================================\n\n");

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
}


#[allow(clippy::needless_collect)]
fn backtracking_correct_program(rules: &Rules) -> Result<(), String> {
    println!("correct");

    let lexer: Lexer = SAMPLE_CORRECT_PROGRAM_0.into();
    let tokens: Vec<_> = lexer.into_iter().map(|it| it.unwrap()).collect();

    let parsed = parse_with_backtracking(rules, tokens.into_iter());

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
fn backtracking_incorrect_program(rules: &Rules) -> Result<(), String> {
    println!("incorrect");

    let lexer: Lexer = SAMPLE_INCORRECT_PROGRAM.into();
    // Parsed successfully above, ok to unwrap.
    let tokens: Vec<_> = lexer.into_iter().map(|it| it.unwrap()).collect();

    let parsed = parse_with_backtracking(rules, tokens.into_iter());

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

fn backtracking(rules: &Rules) -> Result<(), String> {
    println!("\n\n===================================================\n\n");
    backtracking_correct_program(rules)?;

    println!("\n\n===================================================\n\n");
    backtracking_incorrect_program(rules)?;

    println!("\n\n===================================================\n\n");
    Ok(())
}


#[allow(clippy::needless_collect)]
fn recursive_correct_program(rules: &Rules) -> Result<(), String> {
    let lexer: Lexer = SAMPLE_CORRECT_PROGRAM_0.into();

    match recursive_descent_parse(rules, lexer.into_iter()) {
        Ok(tree) => {
            println!("tree:\n{}", display_of(&tree));
            Ok(())
        },
        Err(err) => {
            println!("partial tree:\n{}", display_of(err.partial_tree()));
            Err(err.error().to_string())?
        },
    }
}

#[allow(clippy::needless_collect)]
fn recursive_incorrect_program(rules: &Rules) -> Result<(), String> {
    let lexer: Lexer = SAMPLE_INCORRECT_PROGRAM.into();

    match recursive_descent_parse(rules, lexer.into_iter()) {
        Ok(tree) => {
            println!("tree:\n{}", display_of(&tree));
            Err("expecting failure".to_string())
        },
        Err(err) => {
            println!("partial tree:\n{}", display_of(err.partial_tree()));
            Ok(())
        },
    }
}

fn recursive_unparsable_program(rules: &Rules) -> Result<(), String> {
    let lexer: Lexer = SAMPLE_UNPARSABLE_PROGRAM.into();

    match recursive_descent_parse(rules, lexer.into_iter()) {
        Ok(tree) => {
            println!("tree:\n{}", display_of(&tree));
            Err("expecting failure".to_string())
        },
        Err(err) => {
            println!("partial tree:\n{}", display_of(err.partial_tree()));
            println!("error: {}", err.error());
            Ok(())
        },
    }
}

fn recursive(rules: &Rules) -> Result<(), String> {
    println!("\n\n===================================================\n\n");
    recursive_correct_program(rules)?;

    println!("\n\n===================================================\n\n");
    recursive_incorrect_program(rules)?;

    println!("\n\n===================================================\n\n");
    recursive_unparsable_program(rules)?;

    println!("\n\n===================================================\n\n");
    Ok(())
}


#[allow(clippy::needless_collect)]
fn main() -> Result<(), String> {
    // en_log();

    println!("\n\n===================================================\n\n");

    let mut rules: Rules = GRAMMAR.try_into()?;
    rules.eliminate_left_recursions();
    rules.validate()?;
    println!("left-recursion-free: {}", rules);

    println!("\n\n===================================================\n\n");

    backtracking(&rules)?;

    println!("\n\n===================================================\n\n");

    rules.make_ready_for_recursive_decent(128)?;
    rules.is_backtrack_free()?;
    first_follow_start(&rules);
    println!("backtrack-free: {}", rules);

    println!("\n\n===================================================\n\n");

    recursive(&rules)?;

    println!("\n\n");

    Ok(())
}
