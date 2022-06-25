use std::rc::Rc;

use log::info;
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
         x1 = 1 * 30;\
         x2 = x3 / 10;\
         int y;\
         y = x4 + 2;\
         int z;\
         print(\"foo\\\"bar some thing\");\
         z = x5 * y;\
         int x0;\
    }";

    let mut tokens = vec![];
    for token in Lexer::new(program) {
        let token = token?;
        tokens.push(token)
    }

    let r = toylang_v0_rules();

    let tree = parse_inefficiently(tokens, Rc::clone(&r))?;

    info!("grammar: {}", r.borrow());
    info!("tree: {}", tree.borrow());

    Ok(())
}
