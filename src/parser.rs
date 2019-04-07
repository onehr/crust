use crate::lexer;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NodeType {
    Prog(String),
    // TODO: now only support int parameters
    // <function> ::= "int" <id> "(" [ "int" <id> { "," "int" <id> } ] ")" "{" {<block-item>} "}"
    Fn(String, Option<Vec<String>>),
    Stmt(StmtType),
    // <statement> ::= "return" <exp> ";"
    //               | <exp-option> ";"
    //               | "if" "(" <exp> ")" <statement> [ "else" <statement> ]
    //               | "{" { <block-item> } "}
    //               | "for" "(" <exp-option> ";" <exp-option> ";" <exp-option> ")" <statement>
    //               | "for" "(" <declaration> <exp-option> ";" <exp-option> ")" <statement>
    //               | "while" "(" <exp> ")" <statement>
    //               | "do" <statement> "while" <exp> ";"
    //               | "break" ";"
    //               | "continue" ";"
    Block, // <block> ::= <statement> | <declaration>
    Const(i64),
    Var(String),
    ArrayRef(String),  // referencing to array
    AssignNode(String, bool),     // String -> variable name, bool -> true if this is a assign to array element
    UnExp(lexer::TokType),  // Unary Expression
    BinExp(lexer::TokType), // Binary Operator
    Exp,                    // <exp> ::= <id> ["[" <exp> "]"] "=" <exp> | <conditional-exp>
    ExpOption,              // <exp-option> :: <exp> | ""
    ConditionalExp, // <conditional-exp> ::= <logical-or-exp> [ "?" <exp> ":" <conditional-exp> ]
    LogicalOrExp,   // <logical-or-exp> ::= <logical-and-exp> { "||" <logical-and-exp> }
    LogicalAndExp,  // <logical-and-exp> ::= <equality-exp> { "&&" <equality-exp> }
    EqualityExp,    // <EqualityExp> ::= <relational-exp> { ("!="|"==") <relational-exp> }
    RelationalExp, // <relational-exp> ::= <additive-exp> { ("<" | ">" | "<=" | ">=") <additive-exp> }
    AdditiveExp,   // <additive-exp> ::= <term> { ("+" | "-") <term> }
    Term,          // <term> ::= <factor> { ("*" | "/") <factor> }
    Factor,        // <factor> ::= <function-call> | "(" <exp> ")" | <unary_op> <factor> | <int> | <id> "[" <exp> "]" | <id>
    FnCall(String),            // <function-call> ::= id "(" [ <exp> { "," <exp> } ] ")"
    Declare(String, DataType), // <declaration> ::= "int" <id> "[" <int> "]" ";" | "int" <id> [ = <exp> ] ";"
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum DataType {
    I64,          // now int in c was translated in 64 bits int
    Arr64(i64), // int array[len]
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum StmtType {
    Return,
    Exp,
    Conditional(String),
    Compound,
    For,     // kids: exp-opion, exp-option, exp-option
    ForDecl, // kids: declaration, exp, exp-option, statement
    While,   // kids: exp, stmt
    Do,      // kids: stmt, exp
    Break,
    Continue,
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

fn p_conditional_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // <conditional-exp> ::= <logical-or-exp> [ "?" <exp> ":" <conditional-exp> ]
    let mut conditional_exp_node = ParseNode::new();
    conditional_exp_node.entry = NodeType::ConditionalExp;
    // parse <logical-or-exp> first
    let (logical_or_exp_node, pos) = r#try!(p_logical_or_exp(toks, pos));
    conditional_exp_node.child.push(logical_or_exp_node);

    // it's optional if you got a "?"
    if toks[pos] == lexer::TokType::QuestionMark {
        // parse <exp>
        let pos = pos + 1;
        let (exp_node, pos) = r#try!(p_exp(toks, pos));

        if toks[pos] != lexer::TokType::Colon {
            return Err(format!(
                "Expected `:` in conditional expression, but got {:?} at {}",
                toks[pos], pos
            ));
        }
        let pos = pos + 1;
        // parse next <conditonal-exp>
        let (next_conditional_exp_node, pos) = r#try!(p_conditional_exp(toks, pos));
        conditional_exp_node.child.push(exp_node);
        conditional_exp_node.child.push(next_conditional_exp_node);
        return Ok((conditional_exp_node, pos));
    } else {
        return Ok((conditional_exp_node, pos));
    }
}

fn p_exp_opt(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // <exp-option> ::= <exp> | ""
    let mut exp_opt_node = ParseNode::new();
    exp_opt_node.entry = NodeType::ExpOption;
    let res = p_exp(toks, pos);
    match res {
        Ok((exp_node, pos)) => {
            // <exp>
            exp_opt_node.child.push(exp_node);
            return Ok((exp_opt_node, pos));
        }
        Err(_) => {
            // ""
            // no child, means null statement
            return Ok((exp_opt_node, pos));
        }
    }
}

fn p_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in fn: p_exp, with pos:{}", pos);
    // <exp> ::= <id> [ "[" <exp> "]" ] "=" <exp> | <conditional-exp>
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::Exp;

    let tok = &toks[pos];
    match tok {
        lexer::TokType::Identifier(var_name) => {
            // check next token is Assign
            let mut pos = pos + 1;
            let tok = &toks[pos];
            match tok {
                lexer::TokType::Assign => {
                    pos = pos + 1;
                    // something like a = 1
                    let mut assign_node = ParseNode::new();
                    assign_node.entry = NodeType::AssignNode(var_name.to_string(), false); // assign a int variable
                    let (next_exp_node, pos) = r#try!(p_exp(toks, pos));
                    assign_node.child.push(next_exp_node);
                    return Ok((assign_node, pos));
                }
                lexer::TokType::LBracket => {
                    // something like a[<exp>] = 1;
                    let back_pos = pos-1;
                    pos = pos + 1;
                    // parse exp.
                    let (index_node, new_pos) = r#try!(p_exp(toks, pos));
                    pos = new_pos;
                    // parse ']'
                    if toks[pos] != lexer::TokType::RBracket {
                        return Err(format!("Expected ']' for bracket closing, found {:?} at {}", toks[pos], pos));
                    }

                    // try '='
                    pos = pos + 1;
                    if toks[pos] != lexer::TokType::Assign {
                        pos = back_pos;
                        let (conditional_node, pos) = r#try!(p_conditional_exp(toks, pos));
                        exp_node.child.push(conditional_node);
                        return Ok((exp_node, pos));
                    }
                    pos = pos + 1;
                    // try parse exp
                    let mut assign_node = ParseNode::new();
                    assign_node.entry = NodeType::AssignNode(var_name.to_string(), true); // assign to a array element
                    let (res_node, new_pos) = r#try!(p_exp(toks, pos));
                    pos = new_pos;
                    assign_node.child.push(index_node);
                    assign_node.child.push(res_node);
                    return Ok((assign_node, pos));
                }
                _ => {
                    pos = pos - 1;
                    let (conditional_node, pos) = r#try!(p_conditional_exp(toks, pos));
                    exp_node.child.push(conditional_node);
                    return Ok((exp_node, pos));
                }
            }
        }
        _ => {
            // try <conditional-exp>
            let (cond_node, pos) = r#try!(p_conditional_exp(toks, pos));
            exp_node.child.push(cond_node);
            return Ok((exp_node, pos));
        }
    }
}

fn p_fn(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_fn with pos: {}", pos);
    // <function> ::= "int" <id> "(" ")" "{" { <statement> } "}"
    // now add multi-statements support
    if pos >= toks.len() {
        return Err("Out of program length".to_string());
    }
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
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LParen {
        return Err(format!("Expected `(`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;
    // XXX: add void support, now only support int arg list
    let mut arg_list: Vec<String> = Vec::new();
    while pos < toks.len() && toks[pos] != lexer::TokType::RParen {
        // try to parse argument list
        // match int
        match &toks[pos] {
            lexer::TokType::Kwd(lexer::KwdType::Int) => {
                pos = pos + 1;
            }
            _ => {
                return Err(format!("Expected `int`, found {:?} at {}", toks[pos], pos));
            }
        }
        // match identifier
        match &toks[pos] {
            lexer::TokType::Identifier(var_name) => {
                arg_list.push(var_name.to_string());
                pos = pos + 1;
            }
            _ => {
                return Err(format!(
                    "Expected identifier name, found {:?} at {}",
                    toks[pos], pos
                ));
            }
        }
        // match ,
        match &toks[pos] {
            lexer::TokType::Comma => {
                pos = pos + 1;
            }
            lexer::TokType::RParen => {
                continue;
            }
            _ => {
                return Err(format!(
                    "Expected `,` or `)` at the end of one var_name, found {:?} at {}",
                    toks[pos], pos
                ));
            }
        }
        if toks[pos] == lexer::TokType::RParen {
            break;
        }
    }
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
    if arg_list.is_empty() {
        fn_node.entry = NodeType::Fn(fn_name, None);
    } else {
        fn_node.entry = NodeType::Fn(fn_name, Some(arg_list));
    }

    while pos < toks.len() && toks[pos] != lexer::TokType::RBrace {
        let (block_node, tmp_pos) = r#try!(p_block(toks, pos));
        pos = tmp_pos;
        fn_node.child.push(block_node);
    }

    if pos >= toks.len() {
        return Err(format!("Missing `}}`"));
    }
    if toks[pos] != lexer::TokType::RBrace {
        return Err(format!("Expected `}}`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    //println!("out p_fn with pos: {}", pos);
    Ok((fn_node, pos))
}

fn p_declare(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_declare with pos = {}", pos);
    let tok = &toks[pos];
    match tok {
        lexer::TokType::Kwd(lexer::KwdType::Int) => {
            // "int" <id> [ = <exp> ] ";"
            // or "int" <id> "[" <int> "]" ";"
            let pos = pos + 1;

            let tok = &toks[pos];
            match tok {
                lexer::TokType::Identifier(var_name) => {
                    let mut stmt_node = ParseNode::new();
                    stmt_node.entry = NodeType::Declare(var_name.to_string(), DataType::I64);
                    let pos = pos + 1;
                    let tok = &toks[pos];
                    match tok {
                        lexer::TokType::Assign => {
                            // parse exp
                            // e.g. int a = exp;
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
                            // e.g. int var;
                            let pos = pos + 1;
                            return Ok((stmt_node, pos));
                        }
                        lexer::TokType::LBracket => {
                            // array declare
                            // e.g. int a[100];
                            // XXX: now only just support literal array length
                            if cfg!(feature = "debug") {
                                println!("here in p_declare -> LBraket");
                            }
                            let mut declare_node = ParseNode::new();
                            let pos = pos + 1;
                            let tok = &toks[pos];
                            match tok {
                                lexer::TokType::Literal(n) => {
                                    declare_node.entry = NodeType::Declare(var_name.to_string(), DataType::Arr64(*n));
                                    let pos = pos + 1;
                                    let tok = &toks[pos];
                                    if *tok != lexer::TokType::RBracket {
                                        return Err(format!("Expected `]` for array declaration, found {:?} at {}", toks[pos], pos));
                                    }

                                    let pos = pos + 1;
                                    let tok = &toks[pos];
                                    if *tok != lexer::TokType::Semicolon {
                                        return Err(format!("Expected `;` at end of array declaration, found {:?} at {}", toks[pos], pos));
                                    }
                                    let pos = pos + 1;
                                    if cfg!(feature = "debug") {
                                        println!("got declare_node: {:?}", declare_node);
                                    }
                                    return Ok((declare_node, pos));
                                }
                                _ => {
                                    return Err(format!("Expected Array length `literal`, found {:?} at {}", toks[pos], pos));
                                }
                            }

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
            return Err(format!(
                "Error: Expected type definition `int`, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
}

fn p_block(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    let tok = &toks[pos];
    match tok {
        lexer::TokType::Kwd(lexer::KwdType::Int) => {
            // try to parse declare
            // let mut block_node = ParseNode::new();
            // block_node.entry = NodeType::Block;

            let (declare_node, pos) = r#try!(p_declare(toks, pos));
            // block_node.child.push(declare_node);
            // return Ok((block_node, pos));
            return Ok((declare_node, pos));
        }
        _ => {
            // try to parse statement
            // let mut block_node = ParseNode::new();
            // block_node.entry = NodeType::Block;

            let (stmt_node, pos) = r#try!(p_stmt(toks, pos));
            return Ok((stmt_node, pos));
            // block_node.child.push(stmt_node);
            //return Ok((block_node, pos));
        }
    }
}
fn p_stmt(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in fn : p_stmt, with pos {}", pos);
    let tok = &toks[pos];
    match tok {
        lexer::TokType::LBrace => {
            // "{" { <block-item> } "}"
            let mut pos = pos + 1;
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Compound);

            // try to get some block item
            while toks[pos] != lexer::TokType::RBrace {
                let (block_node, tmp_pos) = r#try!(p_block(toks, pos));
                stmt_node.child.push(block_node);
                pos = tmp_pos;
            }

            // throw "}"
            pos = pos + 1;
            return Ok((stmt_node, pos));
        }
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
            stmt_node.entry = NodeType::Stmt(StmtType::Return);
            stmt_node.child.push(exp_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::If) => {
            // "if" "(" <exp> ")" <statement> [ "else" <statement> ]
            // this is the conditional statement
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Conditional("if".to_string()));
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(`"));
            }
            // try to parse exp
            if cfg!(feature = "debug") {
                println!("here pos = {}", pos);
            }
            let pos = pos + 1;
            let (exp_node, pos) = r#try!(p_exp(toks, pos));
            // println!("pos = {}", pos);
            if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                return Err(format!("Missing `)`"));
            }

            let pos = pos + 1;
            // try to parse statement
            if cfg!(feature = "debug") {
                println!("If: parse stmt from pos = {}, tok: {:?}", pos, toks[pos]);
            }
            let (clause_1_node, pos) = r#try!(p_stmt(toks, pos));
            stmt_node.child.push(exp_node);
            stmt_node.child.push(clause_1_node);

            // if has 'else'
            // println!("SHOULD BE HERE , POS = {}", pos);
            if pos < toks.len() && toks[pos] == lexer::TokType::Kwd(lexer::KwdType::Else) {
                // try to parse statement 2
                let pos = pos + 1;
                let (clause_2_node, pos) = r#try!(p_stmt(toks, pos));
                stmt_node.child.push(clause_2_node);
                return Ok((stmt_node, pos));
            } else {
                return Ok((stmt_node, pos));
            }
        }
        lexer::TokType::Kwd(lexer::KwdType::For) => {
            // "for" "(" <exp-option> ";" <exp-option> ";" <exp-option> ")" <statement>
            // "for" "(" <declaration> <exp-option> ";" <exp-option> ")" <statement>
            let mut stmt_node = ParseNode::new();
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(`"));
            }
            let pos = pos + 1;
            // try to parse declaration
            let decl_res = p_declare(toks, pos);
            match decl_res {
                Ok((declare_node, pos)) => {
                    // "for" "(" <declaration> <exp-option> ";" <exp-option> ")" <statement>
                    stmt_node.child.push(declare_node);
                    stmt_node.entry = NodeType::Stmt(StmtType::ForDecl);

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                        return Err(format!("Missing `;` needed by For"));
                    }
                    let pos = pos + 1;

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);
                    if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                        return Err(format!("Missing `)` needed by For"));
                    }
                    let pos = pos + 1;
                    if cfg!(feature = "debug") {
                        println!("pos: {} tok: {:?} before compound layer", pos, toks[pos]);
                    }
                    let mut compound_layer_node = ParseNode::new();
                    compound_layer_node.entry = NodeType::Stmt(StmtType::Compound);
                    let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
                    compound_layer_node.child.push(next_stmt_node);
                    stmt_node.child.push(compound_layer_node);
                    return Ok((stmt_node, pos));
                }
                Err(_) => {
                    // "for" "(" <exp-option> ";" <exp-option> ";" <exp-option> ")" <statement>
                    stmt_node.entry = NodeType::Stmt(StmtType::For);
                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                        return Err(format!("Missing `;` needed by for"));
                    }
                    let pos = pos + 1;

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                        return Err(format!("Missing `;` needed by for"));
                    }
                    let pos = pos + 1;

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                        return Err(format!("Missing `)` needed by for"));
                    }
                    let pos = pos + 1;
                    let mut compound_layer_node = ParseNode::new();
                    let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
                    compound_layer_node.child.push(next_stmt_node);
                    compound_layer_node.entry = NodeType::Stmt(StmtType::Compound);
                    stmt_node.child.push(compound_layer_node);
                    return Ok((stmt_node, pos));
                }
            }
        }
        lexer::TokType::Kwd(lexer::KwdType::While) => {
            // "while" "(" <exp> ")" <statement>
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::While);
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(` needed by While"));
            }

            let pos = pos + 1;
            let (exp_node, pos) = r#try!(p_exp(toks, pos));
            stmt_node.child.push(exp_node);
            if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                return Err(format!("Missing `)`"));
            }
            let pos = pos + 1;

            let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
            stmt_node.child.push(next_stmt_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Do) => {
            // "do" <statement> "while" "(" <exp> ")" ";"
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Do);
            let pos = pos + 1;
            let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
            stmt_node.child.push(next_stmt_node);
            // parse while
            if pos >= toks.len() || toks[pos] != lexer::TokType::Kwd(lexer::KwdType::While) {
                return Err(format!("Missing `while` needed by do"));
            }
            let pos = pos + 1;

            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(` needed by do"));
            }
            let pos = pos + 1;

            let (exp_node, pos) = r#try!(p_exp_opt(toks, pos));

            if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                return Err(format!("Missing `)` needed by do"));
            }
            let pos = pos + 1;

            if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                return Err(format!("Missing `;` needed by do"));
            }
            let pos = pos + 1;

            stmt_node.child.push(exp_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Continue) => {
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Continue);
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                return Err(format!("Missing `;` needed by continue"));
            }
            let pos = pos + 1;
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Break) => {
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Break);
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                return Err(format!("Missing `;` needed by break"));
            }
            let pos = pos + 1;
            return Ok((stmt_node, pos));
        }
        _ => {
            // try to parse exp-option;
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Exp);
            //let pos = pos + 1;
            let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));

            let tok = &toks[pos];
            if *tok != lexer::TokType::Semicolon {
                return Err(format!("Expected `;`, found {:?} at {}", toks[pos], pos));
            }
            let pos = pos + 1;
            stmt_node.child.push(exp_opt_node);
            return Ok((stmt_node, pos));
        }
    }
}

fn p_factor(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    if cfg!(feature = "debug") {
        println!("in p_factor with pos: {}, tok = {:?}", pos, toks[pos]);
    }
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
            if cfg!(feature = "debug") {
                println!("here\n");
            }
            if pos < toks.len() && toks[pos] == lexer::TokType::LParen {
                // Factor -> FnCall
                let mut factor_node = ParseNode::new();
                pos = pos - 1;
                factor_node.entry = NodeType::Factor;
                let (fn_call_node, pos) = r#try!(p_fn_call(toks, pos));
                factor_node.child.push(fn_call_node);
                return Ok((factor_node, pos));
            } else if pos < toks.len() && toks[pos] == lexer::TokType::LBracket {
                // Factor -> Array referencing
                let mut factor_node = ParseNode::new();
                pos = pos - 1;
                factor_node.entry = NodeType::Factor;
                let (arr_ref_node, pos) = r#try!(p_arr_ref(toks, pos));
                factor_node.child.push(arr_ref_node);
                return Ok((factor_node, pos));
            } else {
                // Factor -> Var
                let mut var_node = ParseNode::new();
                let mut factor_node = ParseNode::new();
                var_node.entry = NodeType::Var(var_name.to_string());
                factor_node.entry = NodeType::Factor;
                factor_node.child.push(var_node);
                // println!("out p_factor with pos: {}", pos);
                return Ok((factor_node, pos));
            }
        }
        _ => Err(format!("Factor rule not allowed.")),
    }
}

fn p_arr_ref(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // array reference ::= <id> "[" <exp> "]"
    let mut arr_ref_node = ParseNode::new();
    let mut var_name = String::new();
    match &toks[pos] {
        lexer::TokType::Identifier(name) => {
            var_name = name.to_string();
        }
        _ => {
            return Err(format!(
                "Expected array identifier, foudn {:?} at {}", toks[pos], pos));
        }
    }
    arr_ref_node.entry = NodeType::ArrayRef(var_name);

    let mut pos = pos + 1;
    // match '['
    match toks[pos] {
        lexer::TokType::LBracket => {
            pos = pos + 1;
        }
        _ => {
            return Err(format!("Expected `[` needed by array referencing, found {:?} at {}", toks[pos], pos));
        }
    }

    let (exp_node, new_pos) = r#try!(p_exp(toks, pos));
    arr_ref_node.child.push(exp_node);
    match toks[new_pos] {
        lexer::TokType::RBracket => {
            pos = new_pos + 1;
        }
        _ => {
            return Err(format!("Expected ']' needed by array referencing, found {:?} at {}", toks[pos], pos));
        }
    }
    return Ok((arr_ref_node, pos));
}
fn p_fn_call(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    // <function-call> ::= id "(" [ <exp> { "," <exp> } ] ")"
    //println!("in fn p_fn_call");
    let mut fn_call_node = ParseNode::new();
    let mut fn_name = String::new();
    match &toks[pos] {
        lexer::TokType::Identifier(name) => {
            fn_name = name.to_string();
        }
        _ => {
            return Err(format!(
                "Expected function name, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
    fn_call_node.entry = NodeType::FnCall(fn_name);
    let mut pos = pos + 1;
    // match '('
    match toks[pos] {
        lexer::TokType::LParen => {
            pos = pos + 1;
        }
        _ => {
            return Err(format!(
                "Expected `(` needed by function call, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
    while pos < toks.len() && toks[pos] != lexer::TokType::RParen {
        // try to parse argument exp
        let (exp_node, new_pos) = r#try!(p_exp(toks, pos));
        fn_call_node.child.push(exp_node);
        pos = new_pos;

        // match ,
        match &toks[pos] {
            lexer::TokType::Comma => {
                pos = pos + 1;
            }
            lexer::TokType::RParen => {
                continue;
            }
            _ => {
                return Err(format!(
                    "Expected `,` or `)` at the end of exp, found {:?} at {}",
                    toks[pos], pos
                ));
            }
        }
        if toks[pos] == lexer::TokType::RParen {
            break;
        }
    }
    pos = pos + 1;
    return Ok((fn_call_node, pos));
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
    let mut prog_node = ParseNode::new();
    prog_node.entry = NodeType::Prog(c_src_name.to_string());
    let mut pos = 0;
    // now we need to add support for global variables
    while pos < toks.len() {
        // try to parse global variable declaration
        let p_res = p_declare(&toks, pos);
        match p_res {
            Ok((decl_node, new_pos)) => {
                pos = new_pos;
                prog_node.child.push(decl_node);
            }
            Err(_) => {
                // try to parse fn definition
                if cfg!(feature = "debug") {
                    println!("try to parse fn definition");
                }
                let (fn_node, new_pos) = r#try!(p_fn(&toks, pos));
                prog_node.child.push(fn_node);
                pos = new_pos;
            }
        }
    }

    return Ok(prog_node);
}

pub fn print(tree: &ParseNode, idt: usize) -> String {
    let mut idt_prefix = String::new();
    for _i in 0..idt {
        idt_prefix = idt_prefix + "    ";
    }
    match &tree.entry {
        NodeType::ArrayRef(var_name) => format!(
            "{}n_type: ArrayRef, var_name : {}, [\n{}\n{}]",
            idt_prefix, var_name,
            print(tree.child.get(0).unwrap(), idt + 1),
            idt_prefix
        ),
        NodeType::Factor => format!(
            "{}n_type: Factor, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Factor Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::AssignNode(var_name, flag) => {
            match flag {
                false => {
                    format!(
                        "{}n_type: AssignNode, Var: {} [\n{}\n{}]",
                        idt_prefix,
                        var_name,
                        print(
                            tree.child.get(0).expect("Assign Node has no child"),
                            idt + 1
                        ),
                        idt_prefix
                    )
                }
                true => {
                    // assign to array
                    format!(
                        "{}n_type: AssignNode array: {} [\n{}\n{}\n{}]",
                        idt_prefix,
                        var_name,
                        print(tree.child.get(0).expect("Assign to array Node has no index node "), idt + 1),
                        print(tree.child.get(1).expect("Assign to array node has no rhs"), idt + 1),
                        idt_prefix,
                    )
                }
            }
        },
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
        NodeType::Prog(prog_name) => {
            let mut prog_body = String::new();
            for it in tree.child.iter() {
                prog_body.push_str(&print(it, idt + 1));
                prog_body.push_str("\n");
            }
            format!(
                "{}n_type: Prog, Name:{} [\n{}\n{}]",
                idt_prefix, prog_name, prog_body, idt_prefix
            )
        }
        NodeType::FnCall(fn_name) => {
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
                "{}n_type: FnCall, Name: {} exp_list: [\n{}\n{}]",
                idt_prefix, fn_name, tmp, idt_prefix
            )
            // list of exp
        }
        NodeType::Fn(fn_name, vars) => {
            let mut tmp = String::new();
            let mut inc = 0;
            for it in tree.child.iter() {
                if inc > 0 {
                    tmp.push_str("\n");
                }
                tmp.push_str(&print(it, idt + 1));
                inc = inc + 1;
            }
            let mut var_list_string = String::new();
            match vars {
                Some(var_list) => {
                    for var in var_list {
                        var_list_string.push_str(" ");
                        var_list_string.push_str(var);
                        var_list_string.push_str(" ");
                    }
                }
                None => {}
            }
            format!(
                "{}n_type: Fn, Name: {} var_list: [{}]\n\
                 {}[\n{}\n{}]",
                idt_prefix, fn_name, var_list_string, idt_prefix, tmp, idt_prefix
            )
        }
        NodeType::Declare(var_name, t) => {
            match t {
                DataType::I64 => {
                    if tree.child.is_empty() {
                        format!("{}n_type: Declare, type: Int var_name: {}", idt_prefix, var_name)
                    } else {
                        format!(
                            "{}n_type: Declare, type: Int var_name: {}, [\n{}\n{}]",
                            idt_prefix,
                            var_name,
                            print(
                                tree.child.get(0).expect("Declare Node has no child"),
                                idt + 1
                            ),
                            idt_prefix
                        )
                    }
                }
                DataType::Arr64(len) => {
                    format!(
                        "{}n_type: Declare, type: Array  var_name: {}, length: {}",
                        idt_prefix,
                        var_name,
                        len,
                    )
                }
            }
        }
        NodeType::ConditionalExp => {
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
                "{}n_type: Stmt::ConditionalExp, [\n{}\n{}]",
                idt_prefix, tmp, idt_prefix
            )
        }
        NodeType::Stmt(stmt) => match stmt {
            StmtType::For => {
                let exp_opt_1 = print(tree.child.get(0).expect("No exp1 in for"), idt + 1);
                let exp_opt_2 = print(tree.child.get(1).expect("No exp2 in for"), idt + 1);
                let exp_opt_3 = print(tree.child.get(2).expect("No exp3 in for"), idt + 1);
                let stmt = print(tree.child.get(3).expect("No stmt in for"), idt + 1);
                format!(
                    "{}n_type: Stmt:ForDeclare, [\n\
                     {}declare: [\n{}\n{}]\n\
                     {}exp1:    [\n{}\n{}]\n\
                     {}exp2:    [\n{}\n{}]\n\
                     {}stmt:    [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    exp_opt_1,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_2,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_3,
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::ForDecl => {
                let d = print(tree.child.get(0).expect("No declaration in for"), idt + 1);
                let exp_opt_1 = print(tree.child.get(1).expect("No exp1 in for"), idt + 1);
                let exp_opt_2 = print(tree.child.get(2).expect("No exp2 in for"), idt + 1);
                let stmt = print(tree.child.get(3).expect("No stmt in for"), idt + 1);

                format!(
                    "{}n_type: Stmt:ForDeclare, [\n\
                     {}declare: [\n{}\n{}]\n\
                     {}exp1:    [\n{}\n{}]\n\
                     {}exp2:    [\n{}\n{}]\n\
                     {}stmt:    [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    d,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_1,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_2,
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::Do => {
                let stmt = print(tree.child.get(0).expect("No stmt in do"), idt + 1);
                let exp = print(tree.child.get(1).expect("No exp in do"), idt + 1);
                format!(
                    "{}n_type: Stmt:Do, [\n\
                     {}           stmt: [\n{}\n{}]\n\
                     {}           exp:  [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix,
                    exp,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::While => {
                let exp = print(tree.child.get(0).expect("No stmt in do"), idt + 1);
                let stmt = print(tree.child.get(1).expect("No exp in do"), idt + 1);
                format!(
                    "{}n_type: Stmt:While, [\n\
                     {}               exp: [\n{}\n{}]\n\
                     {}              stmt:  [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    exp,
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::Continue => format!("{}n_type: Continue", idt_prefix),
            StmtType::Break => format!("{}n_type: Break", idt_prefix),
            StmtType::Return => format!(
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
            StmtType::Exp => format!(
                "{}n_type: Stmt::Exp, [\n{}\n{}]",
                idt_prefix,
                print(
                    tree.child.get(0).expect("Statement::Exp Node has no child"),
                    idt + 1
                ),
                idt_prefix
            ),
            StmtType::Conditional(op) => {
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
                    "{}n_type: Stmt::Conditional, Op: {} [\n{}\n{}]",
                    idt_prefix, op, tmp, idt_prefix
                )
            }
            StmtType::Compound => {
                let mut tmp = String::new();
                let mut inc = 0;

                for it in tree.child.iter() {
                    if inc > 0 {
                        tmp.push_str("\n");
                    }
                    tmp.push_str(&print(it, idt + 1));
                    inc += 1;
                }
                format!(
                    "{}n_type: Stmt::Compound, [\n{}\n{}]",
                    idt_prefix, tmp, idt_prefix
                )
            }
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
        NodeType::ExpOption => match tree.child.len() {
            0 => format!("{}n_type: ExpOption", idt_prefix),
            1 => format!(
                "{}n_type: ExpOption, [\n{}\n{}]",
                idt_prefix,
                print(tree.child.get(0).expect("ExpOption has no child"), idt + 1),
                idt_prefix
            ),
            _ => panic!(format!(
                "ExpOption can only have 0 or 1 child node, but found {} child node",
                tree.child.len()
            )),
        },
        NodeType::Exp => format!(
            "{}n_type: Exp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Block => format!(
            "{}n_type: Block, [\n{}\n{}]",
            idt_prefix,
            print(tree.child.get(0).expect("Block node has no child"), idt + 1),
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
