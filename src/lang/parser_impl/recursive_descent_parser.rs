use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Peekable;
use std::rc::Rc;

use log::trace;

use crate::lang::lexer::token::Token;
use crate::lang::lexer::token::TokenKind;
use crate::lang::lexer::v0::LexerResult;
use crate::lang::parser::node::Node;
use crate::lang::parser::node::ParseError;
use crate::lang::parser::node::ParseResult;
use crate::lang::parser::rule::RulePart;
use crate::lang::parser::rules::Rules;

pub fn recursive_descent_parse<'a, T: Iterator<Item = LexerResult<'a>>>(
    rules: &Rules,
    tokens: T,
) -> ParseResult<'a> {
    RecursiveDescentParser::new(rules, tokens.peekable()).parse_s()
}


struct RecursiveDescentParser<'a, 'b, T: Iterator<Item = LexerResult<'a>>> {
    rules: &'b Rules,

    first_set: HashMap<String, Vec<TokenKind>>,
    follow_set: HashMap<String, Vec<TokenKind>>,

    tokens: Peekable<T>,
    focus: Rc<RefCell<Node<'a>>>,
}

impl<'a, 'b, T: Iterator<Item = LexerResult<'a>>> RecursiveDescentParser<'a, 'b, T> {
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

    fn err_rule(
        &mut self,
        this_rule: &str,
    ) -> ParseResult<'a> {
        let start_tokens = {
            let mut start_tokens = self.first_set[this_rule]
                .iter()
                .map(|it| it.name().to_string())
                .collect::<Vec<_>>();
            start_tokens.sort();
            start_tokens.join(", ")
        };

        let has_epsilon = self
            .rules
            .get_rule_by_name(this_rule)
            .borrow()
            .has_epsilon();

        if has_epsilon {
            let follow = {
                let mut follow = self.follow_set[this_rule]
                    .iter()
                    .map(|it| it.name().to_string())
                    .collect::<Vec<_>>();
                follow.sort();
                follow.join(", ")
            };

            if !self.has_peek() {
                self._err(format!(
                    "rule: {} /// unexpected end of input, expecting one of tokens: {} /// OR because of epsilon one of: {}",
                    this_rule,
                    start_tokens,
                    follow,
                ))
            }
            else {
                let err = format!(
                    "rule: {} /// unexpected token, expecting one of tokens: {} /// OR because of epsilon one of: {}, got: {}",
                    this_rule,
                    start_tokens,
                    follow,
                    self.peek().unwrap(),
                );
                self._err(err)
            }
        }
        else if self.has_peek() {
            let err = format!(
                "rule: {} /// unexpected token, expecting one of tokens: {} got: {}",
                this_rule,
                start_tokens,
                self.peek().unwrap(),
            );
            self._err(err)
        }
        else {
            self._err(format!(
                "rule: {} /// unexpected end of input, expecting one of tokens: {}",
                this_rule, start_tokens,
            ))
        }
    }


    fn pop_to_parent(&mut self) {
        trace!(
            "popping to parent, we are at: {}",
            self.focus.borrow().rule_part().name()
        );
        let parent = Rc::clone(self.focus.borrow().parent().as_ref().unwrap());
        self.focus = parent;
    }

    fn pop_to_root(&mut self) {
        trace!(
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

    fn peek(&mut self) -> Result<&Token<'a>, String> {
        match self.tokens.peek() {
            None => {
                panic!("peek called while no more token is remaining")
            },
            Some(peek) => match peek {
                Ok(peek) => Ok(peek),
                Err(err) => Err(format!(
                    "lexer error, position: {} line: {}, error: {}",
                    err.position, err.line, err.error
                )),
            },
        }
    }

    fn peek_is_in(
        &mut self,
        expecting: &[TokenKind],
    ) -> bool {
        self.has_peek() && expecting.contains(&self.peek().unwrap().token_kind)
    }

    fn peek_is_in_rule_first(
        &mut self,
        rule_name: &str,
    ) -> bool {
        if !self.has_peek() {
            false
        }
        else {
            let tk = self.peek().unwrap().token_kind;
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
            let tk = self.peek().unwrap().token_kind;
            self.follow_set[rule_name].contains(&tk)
        }
    }

    fn peek_is(
        &mut self,
        tk: TokenKind,
    ) -> bool {
        self.peek_is_in(&[tk])
    }


    fn match_tk(
        &mut self,
        expecting: TokenKind,
    ) -> ParseResult<'a> {
        if !self.has_peek() {
            return self._err(format!(
                "unexpected end of input, expecting: {}, got nothing",
                expecting,
            ));
        }

        match self.peek() {
            Ok(_) => {},
            Err(err) => {
                self.pop_to_root();
                return Err(ParseError::new(&self.focus, err));
            },
        }

        trace!(
            "match tk, expecting: {}, current: {}",
            expecting.name(),
            self.peek().unwrap().text
        );

        if self.peek().unwrap().token_kind == expecting {
            let node = self.node_by_token_kind(expecting);
            node.borrow_mut()
                .set_token(self.tokens.next().unwrap().unwrap());
            self.focus.borrow_mut().append_child(&node);
        }
        else {
            let err = format!(
                "unexpected token kind, expecting: {}, got: {}",
                expecting,
                self.peek().unwrap(),
            );
            return self._err(err);
        }

        match self.peek() {
            Ok(_) => Ok(Rc::clone(&self.focus)),
            Err(err) => {
                self.pop_to_root();
                return Err(ParseError::new(&self.focus, err));
            },
        }
    }


    // ============================================================================================

    fn parse_s(mut self) -> ParseResult<'a> {
        trace!("parsing S");
        let node = self.node_by_rule("S");
        self.focus.borrow_mut().append_child(&node);
        self.focus = node;

        if !self.has_peek() {
            trace!("FIN Without start");
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
            self.err_rule("S")
        }
    }

    fn parse_fn_call(&mut self) -> ParseResult<'a> {
        let my_name = "fn_call";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        self.match_tk(TokenKind::Id)?;
        self.match_tk(TokenKind::LeftParen)?;

        self.parse_args()?;

        self.match_tk(TokenKind::RightParen)?;
        self.match_tk(TokenKind::Semicolon)?;

        self.ok_parent()
    }

    fn parse_args(&mut self) -> ParseResult<'a> {
        let my_name = "args";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is_in_rule_first("arg") {
            self.parse_arg()?;
            self.parse_args_0()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_arg(&mut self) -> ParseResult<'a> {
        let my_name = "arg";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if !self.has_peek() {
            self.err_rule(my_name)
        }
        else if self.peek_is(TokenKind::String) {
            self.match_tk(TokenKind::String)?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Int) {
            self.match_tk(TokenKind::Int)?;
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_args_0(&mut self) -> ParseResult<'a> {
        let my_name = "args__0";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is(TokenKind::Comma) {
            self.match_tk(TokenKind::Comma)?;
            self.parse_args()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_fn_declaration(&mut self) -> ParseResult<'a> {
        let my_name = "fn_declaration";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

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
        let my_name = "params";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is_in_rule_first("param") {
            self.parse_param()?;
            self.parse_params_0()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_param(&mut self) -> ParseResult<'a> {
        let my_name = "param";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        self.match_tk(TokenKind::Id)?;
        self.match_tk(TokenKind::Id)?;

        self.ok_parent()
    }

    fn parse_params_0(&mut self) -> ParseResult<'a> {
        let my_name = "params__0";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is(TokenKind::Comma) {
            self.match_tk(TokenKind::Comma)?;
            self.parse_params()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_statements(&mut self) -> ParseResult<'a> {
        let my_name = "statements";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is_in_rule_first("statement") {
            self.parse_statement()?;
            self.parse_statements_0()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_statement(&mut self) -> ParseResult<'a> {
        let my_name = "statement";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.parse_statement_0()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_first("ret") {
            self.parse_ret()?;
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_statements_0(&mut self) -> ParseResult<'a> {
        let my_name = "statements__0";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.parse_statement_0()?;
            self.parse_statements_0()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_first("ret") {
            self.match_tk(TokenKind::Return)?;
            self.parse_expressions()?;
            self.match_tk(TokenKind::Semicolon)?;
            self.parse_statements_0()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_statement_0(&mut self) -> ParseResult<'a> {
        let my_name = "statement__0";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if !self.has_peek() {
            self.err_rule(my_name)
        }
        else if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.match_tk(TokenKind::Semicolon)?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Equal) {
            self.match_tk(TokenKind::Equal)?;
            self.parse_expressions()?;
            self.match_tk(TokenKind::Semicolon)?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::LeftParen) {
            self.match_tk(TokenKind::LeftParen)?;
            self.parse_args()?;
            self.match_tk(TokenKind::RightParen)?;
            self.match_tk(TokenKind::Semicolon)?;
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_ret(&mut self) -> ParseResult<'a> {
        let my_name = "ret";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        self.match_tk(TokenKind::Return)?;
        self.parse_expressions()?;
        self.match_tk(TokenKind::Semicolon)?;

        self.ok_parent()
    }

    fn parse_expressions(&mut self) -> ParseResult<'a> {
        let my_name = "expressions";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        self.parse_terms()?;
        self.parse_expressions_0()?;

        self.ok_parent()
    }

    fn parse_terms(&mut self) -> ParseResult<'a> {
        let my_name = "terms";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        self.parse_factor()?;
        self.parse_terms_0()?;

        self.ok_parent()
    }

    fn parse_expressions_0(&mut self) -> ParseResult<'a> {
        let my_name = "expressions__0";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if !self.has_peek() {
            self.err_rule(my_name)
        }
        else if self.peek_is(TokenKind::Plus) {
            self.match_tk(TokenKind::Plus)?;
            self.parse_expressions()?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Minus) {
            self.match_tk(TokenKind::Minus)?;
            self.parse_expressions()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_factor(&mut self) -> ParseResult<'a> {
        let my_name = "factor";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if !self.has_peek() {
            self.err_rule(my_name)
        }
        else if self.peek_is(TokenKind::LeftParen) {
            self.match_tk(TokenKind::LeftParen)?;
            self.parse_expressions()?;
            self.match_tk(TokenKind::RightParen)?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Int) {
            self.match_tk(TokenKind::Int)?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Id) {
            self.match_tk(TokenKind::Id)?;
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }

    fn parse_terms_0(&mut self) -> ParseResult<'a> {
        let my_name = "terms__0";
        trace!("parsing {}", my_name);

        self.push_to_rule(my_name);

        if !self.has_peek() {
            self.err_rule(my_name)
        }
        else if self.peek_is(TokenKind::Star) {
            self.match_tk(TokenKind::Star)?;
            self.parse_terms()?;
            self.ok_parent()
        }
        else if self.peek_is(TokenKind::Slash) {
            self.match_tk(TokenKind::Slash)?;
            self.parse_terms()?;
            self.ok_parent()
        }
        else if self.peek_is_in_rule_follow(my_name) {
            self.ok_parent()
        }
        else {
            self.err_rule(my_name)
        }
    }
}
