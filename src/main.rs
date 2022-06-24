use pretty_env_logger::formatted_builder;

use toylang::lang::lexer::Lexer;
use toylang::lang::parser::grammar::toylang_v0_rules;
use toylang::lang::parser_impl::inefficient_parser::parse_inefficiently;

fn main() -> Result<(), String> {
    let mut builder = formatted_builder();
    builder.parse_filters("DEBUG");
    builder.try_init().unwrap();

    let program = "\
    fn my_thing42(int j) {
         int x0;\
         x0 = 1 * 30;\
         x0 = x0 / 10;\
         int y = x0 + 2;\
         print(\"foo\\\"bar some thing\");\
         int z = x0 * y;\
    }";

    let mut tokens = vec![];
    for token in Lexer::new(program) {
        let token = token?;
        // info!("token, {}: {}", token.token_kind.name(), token.text);
        tokens.push(token)
    }

    let r = toylang_v0_rules();
    // info!("grammar: {}", *r.borrow());

    parse_inefficiently(tokens, r)?;

    Ok(())
}
