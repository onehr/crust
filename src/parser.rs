use crate::lexer;

// XXX: now I only try to parse return_2.c
//      So the BNF is only:
//      <program>   ::= <function>
//      <function>  ::= "int" <id> "(" ")" "{" <statement> "}"
//      <statement> ::= "return" <exp> ";"
//      <exp>       ::= <int>
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NodeType {
    Prog(String),
    Fn(String),
    Stmt,
    Const(i64),
    UnExp(lexer::TokType),  // Unary Expression
    BinExp(lexer::TokType), // Binary Operator
    Exp,                    // <exp> ::= <term> { ("+" | "-") <term> }
    Term,                   // <term> ::= <factor> { ("*" | "/") <factor> }
    Factor,                 // <factor> ::= "(" <exp> ")" | <unary_op> <factor> | <int>
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParseNode {
    pub child: Vec<ParseNode>,
    pub entry: NodeType,
}

impl ParseNode {
    pub fn new() -> ParseNode {
        ParseNode {
            child: Vec::new(),
            entry: NodeType::Prog("root".to_string()),
        }
    }
}

fn p_fn(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_fn with pos: {}", pos);
    let tok = &toks[pos];
    if *tok != lexer::TokType::Kwd(lexer::KwdType::Int) {
        return Err(format!("Expected `int`, found {:?} at {}", toks[pos], pos));
    }
    let mut pos = pos + 1;

    let tok = &toks[pos];
    let mut fn_name = String::new();
    match tok {
        lexer::TokType::Identifier(n) => {
            fn_name = n.to_string();
        }
        _ => {
            return Err(format!("Expected function name, but not function name"));
        }
    }
    if *tok != lexer::TokType::Identifier("main".to_string()) {
        return Err(format!("Expected `main`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LParen {
        return Err(format!("Expected `(`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::RParen {
        return Err(format!("Expected `)`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LBrace {
        return Err(format!("Expected `{{`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    let (stmt_node, mut pos) = r#try!(p_stmt(toks, pos));

    let tok = &toks[pos];
    if *tok != lexer::TokType::RBrace {
        return Err(format!("Expected `}}`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let mut fn_node = ParseNode::new();
    fn_node.entry = NodeType::Fn(fn_name);
    fn_node.child.push(stmt_node);
    // println!("out p_fn with pos: {}", pos);
    Ok((fn_node, pos))
}

fn p_stmt(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in fn : p_stmt, with pos {}", pos);
    let tok = &toks[pos];
    if *tok != lexer::TokType::Kwd(lexer::KwdType::Ret) {
        return Err(format!(
            "Expected `return`, found {:?} at {}",
            toks[pos], pos
        ));
    }
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::Exp;
    let pos = pos + 1;
    let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
    exp_node.child.push(exp_child_node);

    let tok = &toks[pos];
    if *tok != lexer::TokType::Semicolon {
        return Err(format!("Expected `;`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let mut stmt_node = ParseNode::new();
    stmt_node.entry = NodeType::Stmt;
    stmt_node.child.push(exp_node);
    // println!("out fn : p_stmt, with pos {}", pos);
    Ok((stmt_node, pos))
}

fn p_factor(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_factor with pos: {}", pos);
    let mut next = &toks[pos];
    let mut pos = pos + 1;

    match next {
        lexer::TokType::LParen => {
            // parse expression inside parens
            /// factor -> exp
            let (exp_node, tmp_pos) = r#try!(p_exp(toks, pos));
            pos = tmp_pos;
            next = &toks[pos];
            pos = pos + 1;
            if *next != lexer::TokType::RParen {
                return Err(format!(
                    "Expected `)` in file:parser.rs, found {:?} at {}",
                    toks[pos], pos
                ));
            }
            let mut factor_node = ParseNode::new();
            factor_node.entry = NodeType::Factor;
            factor_node.child.push(exp_node);
            // println!("out p_factor with pos: {}", pos);
            return Ok((factor_node, pos));
        }
        lexer::TokType::Minus | lexer::TokType::Tilde | lexer::TokType::Exclamation => {
            // factor -> UnExp -> factor
            let mut factor_node = ParseNode::new();
            let mut unexp_node = ParseNode::new();
            factor_node.entry = NodeType::Factor;
            unexp_node.entry = NodeType::UnExp(match next {
                lexer::TokType::Minus => lexer::TokType::Minus,
                lexer::TokType::Tilde => lexer::TokType::Tilde,
                lexer::TokType::Exclamation => lexer::TokType::Exclamation,
                _ => panic!("Something strange"),
            });
            let (next_factor_node, pos) = r#try!(p_factor(toks, pos));
            unexp_node.child.push(next_factor_node);
            factor_node.child.push(unexp_node);
            // println!("out p_factor with pos: {}", pos);
            return Ok((factor_node, pos));
        }
        lexer::TokType::Literal(n) => {
            // Factor -> Const
            let mut const_node = ParseNode::new();
            let mut factor_node = ParseNode::new();
            const_node.entry = NodeType::Const(*n);
            factor_node.entry = NodeType::Factor;
            factor_node.child.push(const_node);
            // println!("out p_factor with pos: {}", pos);
            return Ok((factor_node, pos));
        }
        _ => Err(format!("Factor rule not allowed.")),
    }
}

fn p_term(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_term with pos: {}", pos);
    let mut term_node = ParseNode::new();
    term_node.entry = NodeType::Term;
    /// term -> factor
    let mut pos = pos;
    let (factor_node, tmp_pos) = r#try!(p_factor(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::Multi && *tok != lexer::TokType::Splash {
        term_node.child.push(factor_node);
        pos = pos - 1;
        // println!("1. out p_term with pos: {}", pos);
        return Ok((term_node, pos));
    }
    /// term -> BinExp -> (factor_left, factor_right)
    let mut factor_node = factor_node;
    while *tok == lexer::TokType::Multi || *tok == lexer::TokType::Splash {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Multi => lexer::TokType::Multi,
            lexer::TokType::Splash => lexer::TokType::Splash,
            _ => panic!("in p_term, something went wrong"),
        });

        let (next_factor_node, tmp_pos) = r#try!(p_factor(toks, pos));

        binexp_node.child.push(factor_node);
        binexp_node.child.push(next_factor_node);
        factor_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    term_node.child.push(factor_node);
    pos = pos - 1;
    // println!("2. out p_term with pos: {}", pos);
    return Ok((term_node, pos));
}

fn p_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_exp with pos: {}", pos);
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::Exp;
    /// exp -> term
    let mut pos = pos;
    let (term_node, tmp_pos) = r#try!(p_term(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    if *tok != lexer::TokType::Plus && *tok != lexer::TokType::Minus {
        exp_node.child.push(term_node);
        // println!("1.out p_exp with pos: {}", pos);
        return Ok((exp_node, pos));
    }
    /// exp -> BinExp()
    //peek next token, if it is lexer::TokType::Plus or lexer::TokType::Minus
    let mut term_node = term_node;
    let mut pos = pos;
    while *tok == lexer::TokType::Plus || *tok == lexer::TokType::Minus {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Plus => lexer::TokType::Plus,
            lexer::TokType::Minus => lexer::TokType::Minus,
            _ => panic!("in parser::p_exp, something went wrong"),
        });
        pos = pos + 1;
        let (next_term_node, tmp_pos) = r#try!(p_term(toks, pos));
        pos = tmp_pos;
        binexp_node.child.push(term_node);
        binexp_node.child.push(next_term_node);
        term_node = binexp_node;
        tok = &toks[pos];
    }
    exp_node.child.push(term_node);
    // println!("2. out p_exp with pos: {}", pos);
    return Ok((exp_node, pos));
    // let mut exp_node = ParseNode::new();
    // exp_node.entry = NodeType::Exp;
    // let (term, mut pos) = r#try!(p_term(toks, pos));

    // let mut term_node = ParseNode::new();
    // term_node.entry = NodeType::Term;
    // // XXX: we didn't change the term_node.child, I hope it will change in the while loop,
    // //      but I don't whether it will, so need to check out the result
    // exp_node.child.push(term_node);
    // let mut next = toks[pos]; // check the next token, but don't pop it off the list yet
    // if next == lexer::TokType::Plus || next == lexer::TokType::Minus {
    //     exp_node.entry = NodeType::BinExp(next);
    // }

    // while next == lexer::TokType::Plus || next == lexer::TokType::Minus {
    //     // there's another term
    //     let op = toks[pos]; pos = pos + 1;
    //     let (next_term, mut pos) = r#try!(p_term(toks, pos));
    //     let mut next_term_node = ParseNode::new();
    //     next_term_node.entry = NodeType::Term;
    //     term_node.child.push(next_term_node);
    //     term_node = next_term_node;
    // }

    // return Ok((exp_node, pos));

    // let tok = &toks[pos];
    // let mut literal: i64 = 0;
    // // make exp_node
    // match tok {
    //     lexer::TokType::Literal(n) => {
    //         literal = *n;
    //         if *tok != lexer::TokType::Literal(literal) {
    //             return Err(format!(
    //                 "Expected `literal({})`, found {:?} at {}",
    //                 literal, toks[pos], pos
    //             ));
    //         }
    //         let mut const_node = ParseNode::new();
    //         const_node.entry = NodeType::Const(literal);
    //         let pos = pos + 1;
    //         return Ok((const_node, pos));
    //     }
    //     lexer::TokType::Minus => {
    //         let mut exp_node = ParseNode::new();
    //         exp_node.entry = NodeType::UnExp(lexer::TokType::Minus);
    //         let pos = pos + 1;
    //         let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
    //         exp_node.child.push(exp_child_node);
    //         return Ok((exp_node, pos));
    //     }
    //     lexer::TokType::Tilde => {
    //         let mut exp_node = ParseNode::new();
    //         exp_node.entry = NodeType::UnExp(lexer::TokType::Tilde);
    //         let pos = pos + 1;
    //         let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
    //         exp_node.child.push(exp_child_node);
    //         return Ok((exp_node, pos));
    //     }
    //     lexer::TokType::Exclamation => {
    //         let mut exp_node = ParseNode::new();
    //         exp_node.entry = NodeType::UnExp(lexer::TokType::Exclamation);
    //         let pos = pos + 1;
    //         let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
    //         exp_node.child.push(exp_child_node);
    //         return Ok((exp_node, pos));
    //     }
    //     _ => {
    //         return Err(format!("Expect expression"));
    //     }
    // }
}

pub fn parse_prog(input: &String, c_src_name: &String) -> Result<ParseNode, String> {
    let toks = r#try!(lexer::lex(&input));

    p_fn(&toks, 0).and_then(|(n, i)| {
        if i == toks.len() {
            let mut prog_node = ParseNode::new();
            prog_node.entry = NodeType::Prog(c_src_name.to_string());
            prog_node.child.push(n);
            Ok(prog_node)
        } else {
            Err(format!(
                "Expected end of input, found {:?} at {}",
                &toks[i], i
            ))
        }
    })
}

pub fn print(tree: &ParseNode, idt: usize) -> String {
    let mut idt_prefix = String::new();
    for i in 0..idt {
        idt_prefix = idt_prefix + "    ";
    }
    match &tree.entry {
        NodeType::Factor => format!(
            "{}NodeType: Factor, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Factor Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::BinExp(Op) => format!(
            "{}NodeType: BinExp, Op: {} [\n{}\n{}\n{}]",
            idt_prefix,
            match Op {
                lexer::TokType::Minus => "-".to_string(),
                lexer::TokType::Plus => "+".to_string(),
                lexer::TokType::Multi => "*".to_string(),
                lexer::TokType::Splash => "/".to_string(),
                _ => panic!("Operator for Binary Expression not supported"),
            },
            print(tree.child.get(0).expect("BinExp no lhs"), idt + 1),
            print(tree.child.get(1).expect("BinExp no rhs"), idt + 1),
            idt_prefix
        ),
        NodeType::Term => format!(
            "{}NodeType: Term, [\n{}\n{}]",
            idt_prefix,
            print(tree.child.get(0).expect("Term Node has no child"), idt + 1),
            idt_prefix
        ),
        NodeType::Prog(prog_name) => format!(
            "{}NodeType: Prog, Name:{} [\n{}\n{}]",
            idt_prefix,
            prog_name,
            print(
                tree.child.get(0).expect("Progam Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Fn(fn_name) => format!(
            "{}NodeType: Fn, Name: {} [\n{}\n{}]",
            idt_prefix,
            fn_name,
            print(
                tree.child.get(0).expect("Function Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Stmt => format!(
            "{}NodeType: Stmt, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Statement Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Exp => format!(
            "{}NodeType: Exp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::UnExp(Op) => format!(
            "{}NodeType: UnExp, Op: {}, [\n{}\n{}]",
            idt_prefix,
            match Op {
                lexer::TokType::Minus => "-".to_string(),
                lexer::TokType::Tilde => "~".to_string(),
                lexer::TokType::Exclamation => "!".to_string(),
                _ => panic!("Operator for Unary Expression not supported"),
            },
            print(
                tree.child
                    .get(0)
                    .expect("Unary Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Const(n) => format!("{}NodeType: Const, Value: {}", idt_prefix, n),
    }
}
