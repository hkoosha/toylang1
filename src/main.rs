use toylang1::lang::lexer::v0::Lexer;

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

fn main() -> Result<(), String> {
    let lexer: Lexer = SAMPLE_PROGRAM.into();
    for token in lexer {
        println!("token: {}", token?.text);
    }

    Ok(())
}
