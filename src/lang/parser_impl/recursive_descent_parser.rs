use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Peekable;
use std::rc::Rc;

use crate::lang::lexer::token::Token;
use crate::lang::lexer::token::TokenKind;
use crate::lang::parser::node::Node;
use crate::lang::parser::node::ParseError;
use crate::lang::parser::node::ParseResult;
use crate::lang::parser::rule::RulePart;
use crate::lang::parser::rules::Rules;

pub fn recursive_descent_parse<'a, T: Iterator<Item = Token<'a>>>(
    rules: &Rules,
    tokens: T,
) -> ParseResult<'a> {
    RecursiveDescentParser::new(rules, tokens.peekable()).parse_s()
}


struct RecursiveDescentParser<'a, 'b, T: Iterator<Item = Token<'a>>> {
    rules: &'b Rules,

    first_set: HashMap<String, Vec<TokenKind>>,
    follow_set: HashMap<String, Vec<TokenKind>>,

    tokens: Peekable<T>,
    focus: Rc<RefCell<Node<'a>>>,
    current_word: Option<Token<'a>>,
}

impl<'a, 'b, T: Iterator<Item = Token<'a>>> RecursiveDescentParser<'a, 'b, T> {
    fn new(
        rules: &'b Rules,
        tokens: T,
    ) -> Self {
        let rule_part: RulePart = rules.rules().first().unwrap().into();
        let root: Node<'a> = Node::new(rule_part, 0);

        Self {
            rules,
            tokens: tokens.peekable(),
            focus: root.into(),
            current_word: None,
            first_set: rules
                .first_set()
                .into_iter()
                .map(|it| (it.0, it.1.into_iter().collect::<Vec<_>>()))
                .collect(),
            follow_set: rules
                .follow_set()
                .into_iter()
                .map(|it| (it.0, it.1.into_iter().collect::<Vec<_>>()))
                .collect(),
        }
    }


    fn ok_parent(&mut self) -> ParseResult<'a> {
        self.pop_to_parent();
        Ok(Rc::clone(&self.focus))
    }

    fn _err(
        &mut self,
        msg: String,
    ) -> ParseResult<'a> {
        self.pop_to_root();

        Err(ParseError::new(&self.focus, msg))
    }

    fn err_token_kind(
        &mut self,
        expecting: &[TokenKind],
    ) -> ParseResult<'a> {
        self._err(format!(
            "unexpected token kind, expecting: {}, got: {}",
            expecting
                .iter()
                .map(|it| it.name())
                .collect::<Vec<_>>()
                .join(", "),
            self.current_word
                .as_ref()
                .map_or_else(|| "EOF".to_string(), |it| it.to_string())
        ))
    }

    fn err_rule(
        &mut self,
        this_rule: &str,
    ) -> ParseResult<'a> {
        let start_tokens = self.first_set[this_rule]
            .iter()
            .map(|it| it.name())
            .collect::<Vec<_>>()
            .join(", ");

        if self
            .rules
            .get_rule_by_name(this_rule)
            .borrow()
            .has_epsilon()
        {
            let follow = &self.follow_set[this_rule]
                .iter()
                .map(TokenKind::name)
                .collect::<Vec<_>>()
                .join(", ");

            if self.current_word.is_none() {
                self._err(format!(
                    "unexpected end of input, expecting one of tokens: {} OR because of epsilon one of: {}",
                    start_tokens,
                    follow,
                ))
            }
            else {
                self._err(format!(
                    "unexpected token, expecting one of tokens: {} OR because of epsilon one of: {}, got: {}",
                    start_tokens,
                    follow,
                    self.current_word.as_ref().unwrap(),
                ))
            }
        }
        else if self.current_word.is_none() {
            self._err(format!(
                "unexpected end of input, expecting one of tokens: {}",
                start_tokens,
            ))
        }
        else {
            self._err(format!(
                "unexpected token, expecting one of tokens: {} got: {}",
                start_tokens,
                self.current_word.as_ref().unwrap(),
            ))
        }
    }


    fn consume_and_next(&mut self) -> Token<'a> {
        let mut current: Option<Token> = None;
        std::mem::swap(&mut self.current_word, &mut current);
        self.current_word = self.tokens.next();
        let c = current.unwrap();
        println!("Token: {}", c.text);
        c
    }

    fn pop_to_parent(&mut self) {
        println!(
            "popping to parent, we are at: {}",
            self.focus.borrow().rule_part().name()
        );
        let parent = Rc::clone(self.focus.borrow().parent().as_ref().unwrap());
        self.focus = parent;
    }

    fn pop_to_root(&mut self) {
        println!(
            "popping to root, we are at: {}",
            self.focus.borrow().rule_part().name()
        );
        while self.focus.borrow().parent().is_some() {
            self.pop_to_parent();
        }
    }

    fn push_to_rule(
        &mut self,
        rule_name: &str,
    ) {
        let node = self.node_by_rule(rule_name);
        self.focus.borrow_mut().append_child(&node);
        self.focus = node;
    }

    fn node_by_rule(
        &mut self,
        rule_name: &str,
    ) -> Rc<RefCell<Node<'a>>> {
        let rule = self.rules.get_rule_by_name(rule_name);
        let node = Node::new_with_parent(
            RulePart::Rule(rule),
            self.focus.borrow().next_num(),
            &self.focus,
        );
        node.into()
    }

    fn node_by_token_kind(
        &mut self,
        token_kind: TokenKind,
    ) -> Rc<RefCell<Node<'a>>> {
        let node = Node::new(RulePart::Token(token_kind), self.focus.borrow().next_num());
        node.into()
    }


    fn has_peek(&mut self) -> bool {
        self.tokens.peek().is_some()
    }

    fn peek(&mut self) -> &Token<'a> {
        self.tokens.peek().unwrap()
    }

    fn peek_is_in(
        &mut self,
        expecting: &[TokenKind],
    ) -> bool {
        self.has_peek() && expecting.contains(&self.peek().token_kind)
    }

    fn peek_is_in_rule_first(
        &mut self,
        rule_name: &str,
    ) -> bool {
        if !self.has_peek() {
            false
        }
        else {
            let tk = self.peek().token_kind;
            self.first_set[rule_name].contains(&tk)
        }
    }

    fn peek_is_in_rule_follow(
        &mut self,
        rule_name: &str,
    ) -> bool {
        if !self.has_peek() {
            false
        }
        else {
            let tk = self.peek().token_kind;
            self.follow_set[rule_name].contains(&tk)
        }
    }


    fn peek_is(
        &mut self,
        tk: TokenKind,
    ) -> bool {
        self.peek_is_in(&[tk])
    }

    fn expecting(
        &self,
        rule_name: &str,
    ) -> Vec<TokenKind> {
        self.first_set[rule_name]
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    }


    fn match_tk(
        &mut self,
        expecting: TokenKind,
    ) -> ParseResult<'a> {
        if self.current_word.is_none() {
            self.current_word = self.tokens.next();
        }

        let word = match &self.current_word {
            None => return self.err_token_kind(&[expecting]),
            Some(word) => word,
        };

        if word.token_kind == expecting {
            let node = self.node_by_token_kind(expecting);
            node.borrow_mut().set_token(self.consume_and_next());
            self.focus.borrow_mut().append_child(&node);
            return Ok(Rc::clone(&self.focus));
        }

        self.err_token_kind(&[expecting])
    }

    // ============================================================================================

    fn parse_s(mut self) -> ParseResult<'a> {
        println!("parsing S");
        let node = self.node_by_rule("S");
        self.focus.borrow_mut().append_child(&node);
        self.focus = node;

        if !self.has_peek() {
            println!("FIN Without start");
            self.pop_to_root();
            Ok(Rc::clone(&self.focus))
        }
        else if self.peek_is_in_rule_first("fn_call") {
            self.parse_fn_call()
        }
        else if self.peek_is_in_rule_first("fn_declaration") {
            self.parse_fn_declaration()
        }
        else {
            println!("err parsing S else");
            let mut expecting_fn = self.expecting("fn_call");
            expecting_fn.extend(self.expecting("fn_declaration"));
            self.err_token_kind(&expecting_fn)
        }
    }

    fn parse_fn_call(&mut self) -> ParseResult<'a> {
        println!("parsing fn_call");
        self.push_to_rule("fn_call");

        self.match_tk(TokenKind::Id)?;
        self.match_tk(TokenKind::LeftParen)?;

        self.parse_args()?;

        self.match_tk(TokenKind::RightParen)?;
        self.match_tk(TokenKind::Semicolon)?;

        self.ok_parent()
    }

    fn parse_args(&mut self) -> ParseResult<'a> {
        println!("parsing args");
        self.push_to_rule("args");

        if self.peek_is_in_rule_first("arg") {
            self.parse_arg()?;
            self.parse_args_0()?;
        }
        else if !self.peek_is_in_rule_follow("args") {
            println!("parsing args epsilon error");
            return self.err_rule("args");
        }
        // else: Epsilon.

        self.ok_parent()
    }

    fn parse_arg(&mut self) -> ParseResult<'a> {
        println!("parsing arg");
        self.push_to_rule("arg");

        if !self.has_peek() {
            println!("parsing arg eof");
            return self.err_token_kind(&[TokenKind::String, TokenKind::Id, TokenKind::Int]);
        }
        else if self.peek_is(TokenKind::String) {
            self.match_tk(TokenKind::String)?;
        }
        else if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
        }
        else if self.peek_is(TokenKind::Int) {
            self.match_tk(TokenKind::Int)?;
        }
        else {
            unreachable!();
        }

        self.ok_parent()
    }

    fn parse_args_0(&mut self) -> ParseResult<'a> {
        println!("parsing arg__0");
        self.push_to_rule("args__0");

        if self.peek_is(TokenKind::Comma) {
            self.match_tk(TokenKind::Comma)?;
            self.parse_args()?;
        }
        else if !self.peek_is_in_rule_follow("args__0") {
            println!("parsing args__0 epsilone error");
            return self.err_rule("args__0");
        }
        // else -> Epsilon

        self.ok_parent()
    }

    fn parse_fn_declaration(&mut self) -> ParseResult<'a> {
        println!("parsing fn_declaration");
        self.push_to_rule("fn_declaration");

        self.match_tk(TokenKind::Fn)?;
        self.match_tk(TokenKind::Id)?;
        self.match_tk(TokenKind::LeftParen)?;

        self.parse_params()?;

        self.match_tk(TokenKind::RightParen)?;
        self.match_tk(TokenKind::LeftBraces)?;

        self.parse_statements()?;

        self.match_tk(TokenKind::RightBraces)?;

        self.ok_parent()
    }

    fn parse_params(&mut self) -> ParseResult<'a> {
        println!("parsing params");
        self.push_to_rule("params");

        if self.peek_is_in_rule_first("param") {
            self.parse_param()?;
            self.parse_params_0()?;
        }
        else if !self.peek_is_in_rule_follow("params") {
            println!("parsing params epsilon error");
            return self.err_rule("params");
        }
        // else -> Epsilon

        self.ok_parent()
    }

    fn parse_param(&mut self) -> ParseResult<'a> {
        println!("parsing param");
        self.push_to_rule("param");

        self.match_tk(TokenKind::Id)?;
        self.match_tk(TokenKind::Id)?;

        self.ok_parent()
    }

    fn parse_params_0(&mut self) -> ParseResult<'a> {
        println!("parsing params__0");
        self.push_to_rule("params__0");

        println!("PEEK: {}", self.peek());
        println!("FOLLOW: {:?}", &self.follow_set["params__0"]);
        if self.peek_is(TokenKind::Comma) {
            self.match_tk(TokenKind::Comma)?;
            self.parse_params()?;
        }
        else if !self.peek_is_in_rule_follow("params__0") {
            println!("parsing params__0 epsilon error");
            return self.err_rule("params__0");
        }
        // else -> Epsilon.

        self.ok_parent()
    }

    fn parse_statements(&mut self) -> ParseResult<'a> {
        println!("parsing statements");
        self.push_to_rule("statements");

        if self.peek_is_in_rule_first("statement") {
            self.parse_statement()?;
            self.parse_statements_0()?;
        }
        else if !self.peek_is_in_rule_follow("statements") {
            println!("parsing statements epsilon error");
            return self.err_rule("statements");
        }
        // else -> Epsilon.

        self.ok_parent()
    }

    fn parse_statement(&mut self) -> ParseResult<'a> {
        println!("parsing statement");
        self.push_to_rule("statement");

        if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.parse_statements_0()?;
        }
        else if self.peek_is_in_rule_first("ret") {
            self.parse_ret()?;
        }
        else {
            println!("parsing statement epsilon error");
            let mut expecting = self.first_set["ret"].clone();
            expecting.push(TokenKind::Id);
            return self.err_token_kind(&expecting);
        }

        self.ok_parent()
    }

    fn parse_statements_0(&mut self) -> ParseResult<'a> {
        println!("parsing statements__0");
        self.push_to_rule("statements__0");

        if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.parse_statement_0()?;
            self.parse_statements_0()?;
        }
        else if self.peek_is_in_rule_first("ret") {
            self.match_tk(TokenKind::Return)?;
            self.parse_expressions()?;
            self.match_tk(TokenKind::Comma)?;
            self.parse_statements_0()?;
        }
        else if !self.peek_is_in_rule_follow("statements__0") {
            println!("parsing statements__0 error");
            return self.err_rule("statements__0");
        }
        // else -> Epsilon.

        self.ok_parent()
    }

    fn parse_statement_0(&mut self) -> ParseResult<'a> {
        println!("parsing statement__0");
        self.push_to_rule("statement__0");

        if !self.has_peek() {
            println!("parsing statement__0 eof error");
            return self.err_token_kind(&[TokenKind::Id, TokenKind::Equal, TokenKind::LeftParen]);
        }
        else if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.match_tk(TokenKind::Semicolon)?;
        }
        else if self.peek_is(TokenKind::Equal) {
            self.match_tk(TokenKind::Equal)?;
            self.parse_expressions()?;
            self.match_tk(TokenKind::Semicolon)?;
        }
        else if self.peek_is(TokenKind::LeftParen) {
            self.match_tk(TokenKind::LeftParen)?;
            self.parse_args()?;
            self.match_tk(TokenKind::RightParen)?;
        }
        else {
            unreachable!();
        }

        self.ok_parent()
    }

    fn parse_ret(&mut self) -> ParseResult<'a> {
        println!("parsing ret");
        self.push_to_rule("ret");

        self.match_tk(TokenKind::Return)?;
        self.parse_expressions()?;
        self.match_tk(TokenKind::Semicolon)?;

        self.ok_parent()
    }

    fn parse_expressions(&mut self) -> ParseResult<'a> {
        println!("parsing expressions");
        self.push_to_rule("expressions");

        self.parse_terms()?;
        self.parse_expressions_0()?;

        self.ok_parent()
    }

    fn parse_terms(&mut self) -> ParseResult<'a> {
        println!("parsing terms");
        self.push_to_rule("expressions");

        self.parse_factor()?;
        self.parse_terms_0()?;

        self.ok_parent()
    }

    fn parse_expressions_0(&mut self) -> ParseResult<'a> {
        println!("parsing expressions__0");
        self.push_to_rule("expressions__0");

        if self.peek_is(TokenKind::Plus) {
            println!("parsing expressions0");
            self.match_tk(TokenKind::Plus)?;
            self.parse_expressions()?;
        }
        else if self.peek_is(TokenKind::Minus) {
            self.match_tk(TokenKind::Minus)?;
            self.parse_expressions()?;
        }
        else if !self.peek_is_in_rule_follow("expressions__0") {
            println!("parsing expressions__0 epsilon error");
            return self.err_rule("expressions__0");
        }
        // else -> Epsilon

        self.ok_parent()
    }

    fn parse_factor(&mut self) -> ParseResult<'a> {
        println!("parsing factor");
        self.push_to_rule("factor");

        if !self.has_peek() {
            println!("parsing factor eof error");
            return self.err_token_kind(&[TokenKind::LeftParen, TokenKind::Int, TokenKind::Id]);
        }
        else if self.peek_is(TokenKind::LeftParen) {
            self.match_tk(TokenKind::LeftParen)?;
            self.parse_expressions()?;
            self.match_tk(TokenKind::RightParen)?;
        }
        else if self.peek_is(TokenKind::Int) {
            self.match_tk(TokenKind::Int)?;
        }
        else if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
        }
        else {
            unreachable!();
        }

        self.ok_parent()
    }

    fn parse_terms_0(&mut self) -> ParseResult<'a> {
        println!("parsing terms__0");
        self.push_to_rule("terms__0");

        if !self.has_peek() {
            println!("parsing terms__0 eof error");
            return self.err_token_kind(&[TokenKind::Star, TokenKind::Slash]);
        }
        else if self.peek_is(TokenKind::Star) {
            self.match_tk(TokenKind::Star)?;
            self.parse_terms()?;
        }
        else if self.peek_is(TokenKind::Slash) {
            self.match_tk(TokenKind::Slash)?;
            self.parse_terms()?;
        }
        else {
            unreachable!();
        }

        self.ok_parent()
    }
}
