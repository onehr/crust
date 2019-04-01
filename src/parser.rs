use crate::lexer;

// XXX: now I only try to parse return_num.c
//      So the BNF is only:
//      <program>   ::= <function>
//      <function>  ::= "int" <id> "(" ")" "{" <statement> "}"
//      <statement> ::= "return" <exp> ";"
//      <exp>       ::= <int>
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NodeType {
    Prog(String),
    Fn(String, Option<Box<NodeType>>), // name,variables
    Stmt(stmt_type),
    Const(i64),
    Var(String),
    AssignNode(String),     // String -> variable name
    UnExp(lexer::TokType),  // Unary Expression
    BinExp(lexer::TokType), // Binary Operator
    Exp,                    // <exp> ::= <id> "=" <exp> | <logical-or-exp>
    LogicalOrExp,           // <logical-or-exp> ::= <logical-and-exp> { "||" <logical-and-exp> }
    LogicalAndExp,          // <logical-and-exp> ::= <equality-exp> { "&&" <equality-exp> }
    EqualityExp,            // <EqualityExp> ::= <relational-exp> { ("!="|"==") <relational-exp> }
    RelationalExp, // <relational-exp> ::= <additive-exp> { ("<" | ">" | "<=" | ">=") <additive-exp> }
    AdditiveExp,   // <additive-exp> ::= <term> { ("+" | "-") <term> }
    Term,          // <term> ::= <factor> { ("*" | "/") <factor> }
    Factor,        // <factor> ::= "(" <exp> ")" | <unary_op> <factor> | <int> | <id>
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum stmt_type {
    Return,
    Declare(String), // String -> variable name
    Exp,
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

fn p_logical_or_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    let mut log_or_exp_node = ParseNode::new();
    log_or_exp_node.entry = NodeType::LogicalOrExp;
    // Parse <logical-and-exp> first

    // <LogicalOrExp> -> <LogicalAndExp>
    let mut pos = pos;
    let (log_and_exp_node, tmp_pos) = r#try!(p_logical_and_exp(toks, pos));
    pos = tmp_pos;
    // peek next node
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::Or {
        // only one child_node
        log_or_exp_node.child.push(log_and_exp_node);
        pos = pos - 1;
        return Ok((log_or_exp_node, pos));
    }

    /// log_or_exp -> BinExp -> (left: logAndExp, right logAndExp)
    let mut lhs = log_and_exp_node;
    while *tok == lexer::TokType::Or {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(lexer::TokType::Or);

        let (rhs, tmp_pos) = r#try!(p_logical_and_exp(toks, pos));

        binexp_node.child.push(lhs);
        binexp_node.child.push(rhs);
        lhs = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    log_or_exp_node.child.push(lhs);
    pos = pos - 1;
    return Ok((log_or_exp_node, pos));
}

fn p_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    //println!("in fn: p_exp, wiit pos:{}", pos);
    // <exp> ::= <id> "=" <exp> | <logical-or-exp>
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::Exp;

    let tok = &toks[pos];
    match tok {
        lexer::TokType::Identifier(var_name) => {
            // check next token is Assign
            let mut pos = pos + 1;
            let tok = &toks[pos];
            if tok != &lexer::TokType::Assign {
                pos = pos - 1;
                let (log_or_exp_node, pos) = r#try!(p_logical_or_exp(toks, pos));
                exp_node.child.push(log_or_exp_node);
                return Ok((exp_node, pos));
                // return Err(format!("Expected `=`, found {:?} at {}", toks[pos], pos));
            }
            pos = pos + 1;
            // something like a = 1
            let mut assign_node = ParseNode::new();
            assign_node.entry = NodeType::AssignNode(var_name.to_string());
            let (next_exp_node, pos) = r#try!(p_exp(toks, pos));
            assign_node.child.push(next_exp_node);
            return Ok((assign_node, pos));
            //exp_node.child.push(next_exp_node);
            //return Ok((exp_node, pos));
        }
        _ => {
            // try <logical-or-exp>
            let (log_or_exp_node, pos) = r#try!(p_logical_or_exp(toks, pos));
            exp_node.child.push(log_or_exp_node);
            return Ok((exp_node, pos));
        }
    }
}
// XXX: now only support one function: which is main
fn p_fn(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_fn with pos: {}", pos);
    // <function> ::= "int" <id> "(" ")" "{" { <statement> } "}"
    // now add multi-statements support
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

    let mut fn_node = ParseNode::new();
    fn_node.entry = NodeType::Fn(fn_name, None);
    let mut tok = &toks[pos];
    while *tok != lexer::TokType::RBrace {
        let (stmt_node, tmp_pos) = r#try!(p_stmt(toks, pos));
        pos = tmp_pos;
        fn_node.child.push(stmt_node);
        tok = &toks[pos];
    }

    let tok = &toks[pos];
    if *tok != lexer::TokType::RBrace {
        return Err(format!("Expected `}}`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    // println!("out p_fn with pos: {}", pos);
    Ok((fn_node, pos))
}

fn p_stmt(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in fn : p_stmt, with pos {}", pos);
    let tok = &toks[pos];
    match tok {
        lexer::TokType::Kwd(lexer::KwdType::Ret) => {
            // "return" <exp> ";"
            let pos = pos + 1;
            let (exp_node, mut pos) = r#try!(p_exp(toks, pos));

            let tok = &toks[pos];
            if *tok != lexer::TokType::Semicolon {
                return Err(format!(
                    "Expected `;` in statement, found {:?} at {}",
                    toks[pos], pos
                ));
            }
            pos = pos + 1;

            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(stmt_type::Return);
            stmt_node.child.push(exp_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Int) => {
            // "int" id [ = <exp> ] ";"
            let pos = pos + 1;

            let tok = &toks[pos];
            match tok {
                lexer::TokType::Identifier(var_name) => {
                    let mut stmt_node = ParseNode::new();
                    stmt_node.entry = NodeType::Stmt(stmt_type::Declare(var_name.to_string()));
                    let pos = pos + 1;
                    let tok = &toks[pos];
                    match tok {
                        lexer::TokType::Assign => {
                            // parse exp
                            let pos = pos + 1;
                            let (exp_node, pos) = r#try!(p_exp(toks, pos));

                            let tok = &toks[pos];
                            if *tok != lexer::TokType::Semicolon {
                                return Err(format!(
                                    "Expected `;`, found {:?} at {}",
                                    toks[pos], pos
                                ));
                            }
                            let pos = pos + 1;
                            stmt_node.child.push(exp_node);
                            return Ok((stmt_node, pos));
                        }
                        lexer::TokType::Semicolon => {
                            // if just declare, but no assignment, just record the var_name
                            let pos = pos + 1;
                            return Ok((stmt_node, pos));
                        }
                        _ => {
                            return Err(format!(
                                "Expected Assignment `;` or `=`, found {:?} at {}",
                                toks[pos], pos
                            ));
                        }
                    }
                }
                _ => {
                    return Err(format!(
                        "Expected identifier name, found {:?} at {}",
                        toks[pos], pos
                    ));
                }
            }
        }
        _ => {
            // try to parse exp;
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(stmt_type::Exp);
            //let pos = pos + 1;
            let (exp_node, pos) = r#try!(p_exp(toks, pos));

            let tok = &toks[pos];
            if *tok != lexer::TokType::Semicolon {
                return Err(format!("Expected `;`, found {:?} at {}", toks[pos], pos));
            }
            let pos = pos + 1;
            stmt_node.child.push(exp_node);
            return Ok((stmt_node, pos));
        }
    }
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
        lexer::TokType::Identifier(var_name) => {
            // Factor -> Var
            let mut var_node = ParseNode::new();
            let mut factor_node = ParseNode::new();
            var_node.entry = NodeType::Var(var_name.to_string());
            factor_node.entry = NodeType::Factor;
            factor_node.child.push(var_node);

            return Ok((factor_node, pos));
        }
        _ => Err(format!("Factor rule not allowed.")),
    }
}

fn p_logical_and_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    let mut logAndExp_node = ParseNode::new();
    logAndExp_node.entry = NodeType::LogicalAndExp;

    // LogicalAndExp -> EqualityExp
    let mut pos = pos;
    let (eq_node, tmp_pos) = r#try!(p_eq_exp(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::And {
        logAndExp_node.child.push(eq_node);
        pos = pos - 1;
        return Ok((logAndExp_node, pos));
    }
    // Term -> BinExp -> (EqualityExp, EqualityExp)
    let mut eq_node = eq_node; // change to mutable
    while *tok == lexer::TokType::And {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(lexer::TokType::And);

        let (rhs, tmp_pos) = r#try!(p_eq_exp(toks, pos));

        binexp_node.child.push(eq_node);
        binexp_node.child.push(rhs);
        eq_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    logAndExp_node.child.push(eq_node);
    pos = pos - 1;
    return Ok((logAndExp_node, pos));
}

fn p_eq_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    let mut eq_node = ParseNode::new();
    eq_node.entry = NodeType::EqualityExp;

    let mut pos = pos;
    let (relational_node, tmp_pos) = r#try!(p_relational_exp(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::NotEqual && *tok != lexer::TokType::Equal {
        eq_node.child.push(relational_node);
        pos = pos - 1;
        return Ok((eq_node, pos));
    }

    let mut relational_node = relational_node;
    while *tok == lexer::TokType::Equal || *tok == lexer::TokType::NotEqual {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Equal => lexer::TokType::Equal,
            lexer::TokType::NotEqual => lexer::TokType::NotEqual,
            _ => panic!("in p_eq_exp, something went wrong"),
        });

        let (next_relational_node, tmp_pos) = r#try!(p_relational_exp(toks, pos));

        binexp_node.child.push(relational_node);
        binexp_node.child.push(next_relational_node);
        relational_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    eq_node.child.push(relational_node);
    pos = pos - 1;
    return Ok((eq_node, pos));
}

fn p_relational_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    let mut relational_node = ParseNode::new();
    relational_node.entry = NodeType::RelationalExp;

    let mut pos = pos;
    let (additive_exp_node, tmp_pos) = r#try!(p_additive_exp(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::Lt
        && *tok != lexer::TokType::Gt
        && *tok != lexer::TokType::GreaterEqual
        && *tok != lexer::TokType::LessEqual
    {
        relational_node.child.push(additive_exp_node);
        pos = pos - 1;
        return Ok((relational_node, pos));
    }

    let mut additive_exp_node = additive_exp_node;
    while *tok == lexer::TokType::Lt
        || *tok == lexer::TokType::Gt
        || *tok == lexer::TokType::GreaterEqual
        || *tok == lexer::TokType::LessEqual
    {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Lt => lexer::TokType::Lt,
            lexer::TokType::Gt => lexer::TokType::Gt,
            lexer::TokType::GreaterEqual => lexer::TokType::GreaterEqual,
            lexer::TokType::LessEqual => lexer::TokType::LessEqual,
            _ => panic!("in p_relational_exp, something went wrong"),
        });
        let (next_additive_exp_node, tmp_pos) = r#try!(p_additive_exp(toks, pos));
        binexp_node.child.push(additive_exp_node);
        binexp_node.child.push(next_additive_exp_node);
        additive_exp_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    relational_node.child.push(additive_exp_node);
    pos = pos - 1;
    return Ok((relational_node, pos));
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

fn p_additive_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_exp with pos: {}", pos);
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::AdditiveExp;
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
    for _i in 0..idt {
        idt_prefix = idt_prefix + "    ";
    }
    match &tree.entry {
        NodeType::Factor => format!(
            "{}n_type: Factor, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Factor Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::AssignNode(var_name) => format!(
            "{}n_type: AssignNode, Var: {} [\n{}\n{}]",
            idt_prefix,
            var_name,
            print(
                tree.child.get(0).expect("Assign Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::BinExp(op) => format!(
            "{}n_type: BinExp, Op: {} [\n{}\n{}\n{}]",
            idt_prefix,
            match op {
                lexer::TokType::Minus => format!("-"),
                lexer::TokType::Plus => format!("+"),
                lexer::TokType::Multi => format!("*"),
                lexer::TokType::Splash => format!("/"),
                lexer::TokType::And => format!("&&"),
                lexer::TokType::Or => format!("||"),
                lexer::TokType::Equal => format!("=="),
                lexer::TokType::NotEqual => format!("!="),
                lexer::TokType::GreaterEqual => format!(">="),
                lexer::TokType::LessEqual => format!("<="),
                lexer::TokType::Lt => format!("<"),
                lexer::TokType::Gt => format!(">"),
                _ => panic!(format!(
                    "Operator `{:?}` for Binary Expression not supported",
                    &op
                )),
            },
            print(tree.child.get(0).expect("BinExp no lhs"), idt + 1),
            print(tree.child.get(1).expect("BinExp no rhs"), idt + 1),
            idt_prefix
        ),
        NodeType::Term => format!(
            "{}n_type: Term, [\n{}\n{}]",
            idt_prefix,
            print(tree.child.get(0).expect("Term Node has no child"), idt + 1),
            idt_prefix
        ),
        NodeType::Prog(prog_name) => format!(
            "{}n_type: Prog, Name:{} [\n{}\n{}]",
            idt_prefix,
            prog_name,
            print(
                tree.child.get(0).expect("Progam Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Fn(fn_name, _) => {
            let mut tmp = String::new();
            let mut inc = 0;
            for it in tree.child.iter() {
                if inc > 0 {
                    tmp.push_str("\n");
                }
                tmp.push_str(&print(it, idt + 1));
                inc = inc + 1;
            }
            format!(
                "{}n_type: Fn, Name: {} [\n{}\n{}]",
                idt_prefix,
                fn_name,
                tmp,
                // print(
                //     tree.child.get(0).expect("Function Node has no child"),
                //     idt + 1
                // ),
                idt_prefix
            )
        }
        NodeType::Stmt(stmt) => match stmt {
            stmt_type::Return => format!(
                "{}n_type: Stmt::Return, [\n{}\n{}]",
                idt_prefix,
                print(
                    tree.child
                        .get(0)
                        .expect("Statement::Return Node has no child"),
                    idt + 1
                ),
                idt_prefix
            ),
            stmt_type::Declare(var_name) => {
                if tree.child.is_empty() {
                    format!(
                        "{}n_type: Stmt::Declare, var_name: {}",
                        idt_prefix, var_name
                    )
                } else {
                    format!(
                        "{}n_type: Stmt::Declare, var_name: {}, [\n{}\n{}]",
                        idt_prefix,
                        var_name,
                        print(
                            tree.child
                                .get(0)
                                .expect("Statement::Declare Node has no child"),
                            idt + 1
                        ),
                        idt_prefix
                    )
                }
            }
            stmt_type::Exp => format!(
                "{}n_type: Stmt::Exp, [\n{}\n{}]",
                idt_prefix,
                print(
                    tree.child.get(0).expect("Statement::Exp Node has no child"),
                    idt + 1
                ),
                idt_prefix
            ),
        },
        NodeType::LogicalOrExp => format!(
            "{}n_type: LogicalOrExp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child
                    .get(0)
                    .expect("Logical_Or_Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::LogicalAndExp => format!(
            "{}n_type: LogicalAndExp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child
                    .get(0)
                    .expect("Logical_And_Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::EqualityExp => format!(
            "{}n_type: EqualityExp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::RelationalExp => format!(
            "{}n_type: RelationalExp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::AdditiveExp => format!(
            "{}n_type: AdditiveExp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Exp => format!(
            "{}n_type: Exp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::UnExp(Op) => format!(
            "{}n_type: UnExp, Op: {}, [\n{}\n{}]",
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
        NodeType::Var(var_name) => format!("{}n_type, Variable, Name : {}", idt_prefix, var_name),
        NodeType::Const(n) => format!("{}n_type: Const, Value: {}", idt_prefix, n),
        _ => panic!(format!(
            "in parser::print, {:?} Node type not implemented",
            &tree.entry
        )),
    }
}
