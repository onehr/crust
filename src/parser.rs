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
    //Exp(i64),
    Exp,
    Const(i64),
    //UnExp((lexer::TokType, Box<NodeType>)),
    UnExp(lexer::TokType),
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
    Ok((fn_node, pos))
}

fn p_stmt(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
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
    Ok((stmt_node, pos))
}

fn p_exp(toks: &Vec<lexer::TokType>, pos: usize) -> Result<(ParseNode, usize), String> {
    let tok = &toks[pos];
    let mut literal: i64 = 0;
    // make exp_node
    match tok {
        lexer::TokType::Literal(n) => {
            literal = *n;
            if *tok != lexer::TokType::Literal(literal) {
                return Err(format!(
                    "Expected `literal({})`, found {:?} at {}",
                    literal, toks[pos], pos
                ));
            }
            let mut const_node = ParseNode::new();
            const_node.entry = NodeType::Const(literal);
            let pos = pos + 1;
            return Ok((const_node, pos));
        }
        lexer::TokType::Minus => {
            let mut exp_node = ParseNode::new();
            exp_node.entry = NodeType::UnExp(lexer::TokType::Minus);
            let pos = pos + 1;
            let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
            exp_node.child.push(exp_child_node);
            return Ok((exp_node, pos));
        }
        lexer::TokType::Tilde => {
            let mut exp_node = ParseNode::new();
            exp_node.entry = NodeType::UnExp(lexer::TokType::Tilde);
            let pos = pos + 1;
            let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
            exp_node.child.push(exp_child_node);
            return Ok((exp_node, pos));
        }
        lexer::TokType::Exclamation => {
            let mut exp_node = ParseNode::new();
            exp_node.entry = NodeType::UnExp(lexer::TokType::Exclamation);
            let pos = pos + 1;
            let (exp_child_node, mut pos) = r#try!(p_exp(toks, pos));
            exp_node.child.push(exp_child_node);
            return Ok((exp_node, pos));
        }
        _ => {
            return Err(format!("Expect expression"));
        }
    }
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
