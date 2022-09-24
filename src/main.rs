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
arg             -> TXT | INT | ID
fn_declaration  -> FN ID ( params ) { statements }
params          -> param , params | param |
param           -> ID ID
statements      -> statement statements | statement |
statement       -> declaration | assignment | fn_call | ret
declaration     -> ID ID ;
assignment      -> ID = expressions ;
expressions     -> terms + expressions | terms - expressions | terms
terms           -> factor * terms | factor / terms | factor
factor          -> ( expressions ) | INT | ID
ret             -> RETURN expressions ;

";

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
            panic!("unexpected error: {}", parse_error)
        },
    }

    Ok(())
}

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

fn main() -> Result<(), String> {
    let mut rules: Rules = GRAMMAR.try_into()?;
    rules.eliminate_left_recursions();
    rules.validate()?;

    println!("\n\n===================================================\n\n");
    correct_program(&rules)?;

    println!("\n\n===================================================\n\n");
    incorrect_program(&rules)?;

    println!("\n\n===================================================\n\n");

    println!("RULES: {}", rules);

    let first = rules.find_first_set();
    for (name, f) in first {
        println!("first of {} => {:?}", name, f);
    }

    println!("\n\n");

    Ok(())
}
