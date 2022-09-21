use toylang1::lang::lexer::v0::Lexer;
use toylang1::lang::parser::node::display_of;
use toylang1::lang::parser::rules::Rules;
use toylang1::lang::parser_impl::backtracking_parser::parse;

const SAMPLE_PROGRAM: &str = "\
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

fn main() -> Result<(), String> {
    let lexer: Lexer = SAMPLE_PROGRAM.into();
    for token in lexer {
        token?;
    }

    let lexer: Lexer = SAMPLE_PROGRAM.into();
    // Parsed successfully above, ok to unwrap.
    let tokens: Vec<_> = lexer.into_iter().map(|it| it.unwrap()).collect();

    let mut rules: Rules = GRAMMAR.try_into()?;
    rules.eliminate_left_recursions();
    rules.is_valid()?;

    let parsed = parse(&rules, tokens.into_iter())?;
    let display = display_of(&parsed);
    println!("parsed: {}", display);

    Ok(())
}
