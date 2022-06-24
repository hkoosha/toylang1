/*    fn parse(&mut self, tree: &mut Node<'a>) -> Result<(), String> {
    if self.focus.as_ref().is_some() {
        println!(
            "focus: {}  <==> tk: {}, t: {}",
            &self.focus.as_ref().unwrap().borrow().name(),
            &self.tokens[self.tokens_i].token_kind,
            &self.tokens[self.tokens_i].text,
        );
    }

    return if self.focus.is_none() {
        Err("no focus".to_string())
    }
    else if self.focus.as_ref().unwrap().borrow().is_non_terminal() {
        let focus = Rc::clone(&*self.focus.as_ref().unwrap());
        let focus_borrow = &*focus.borrow();
        match focus_borrow {
            Rule::Terminal(_) => unreachable!(),
            Rule::Expandable { rules, .. } => {
                for rule in rules.iter().rev() {
                    println!("expandable sub-rule: {}", rule.borrow().name());
                    let child = Node::child(rule);
                    tree.children.push(child);
                }
                self.focus = Some(Rc::clone(&tree.children.last().unwrap().rule));
                self.parse(tree)
            }
            Rule::Alternative { rules, .. } => {
                for alternative in rules {
                    println!("alternative: {}", alternative.borrow().name());
                    self.focus = Some(Rc::clone(&alternative));
                }
                self.parse(tree)
            }
        }
    }
    else if self
        .focus
        .as_ref()
        .unwrap()
        .borrow()
        .matches(&self.tokens[self.tokens_i].token_kind)
    {
        println!("match");
        self.tokens_i += 1;
        // self.focus = self.stack.pop();
        todo!("set self focus");
        Ok(())
    }
    else {
        Err("no match".to_string())
    };
}*/
