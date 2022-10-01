A toy language, from scratch, based on the book [Engineering a Compiler][engineering_a_compiler].

### Currently implemented

- A lexer, with predefined token kinds (defined in token.rs).
- A backtracking parser for an arbitrary grammar, but currently depends on the
  lexer and it's predefined token kinds.
- A backtrack-free, recursive descent parser for an arbitrary grammar, but
  currently depends on the lexer and it's predefined token kinds.

## Rules parser

A sample grammar, simply defined as a string and parsed into grammar. The
grammar can have left recursion, the `Rules` parser can eliminate left
recursions.

It can also (almost) fix the grammar, so it becomes backtrack-free by
eliminating common left prefix from production definitions.

```
S               -> fn_call_or_decl , S | fn_call_or_decl |
fn_call_or_decl -> fn_call | fn_declaration
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
```

### Example Program

A sample syntactically correct program for grammar above (semantically it will
be wrong, after implementing later stages of the compiler due to missing type
definitions for x1, x2, ... ):

```
fn my_thing42(int j, string q) {
    x1 = 1 * 30;
    x2 = x3 / 10;
    int y;
    y = x4 + 2;
    int z;
    print("foo\"bar \\some thing");
    z = x5 * y;
    print(z);
    int x0;
    return x0 + 0;
};
```

[engineering_a_compiler]: https://www.elsevier.com/books/engineering-a-compiler/cooper/978-0-12-815412-0
