use crate::lexer;
use std::iter::Peekable;

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
    Exp(i64),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParseNode {
    child: Vec<ParseNode>,
    entry: NodeType,
}

impl ParseNode {
    pub fn new() -> ParseNode {
        ParseNode {
            child: Vec::new(),
            entry: NodeType::Prog("root".to_string()),
        }
    }
}

fn p_fn(
    toks: &Vec<lexer::TokType>,
    pos: usize,
) -> Result<(ParseNode, usize), String> {
    //println!("into function p_fn");
    let tok = &toks[pos];
    if *tok != lexer::TokType::Kwd(lexer::KwdType::Int) {
        //Err(format!("Expected `int`, found {:?} at {}", toks[pos], pos));
    }
    let mut pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::Identifier("main".to_string()) {
        //Err(format!("Expected `main`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LParen {
        //Err(format!("Expected `(`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::RParen {
        //Err(format!("Expected `)`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LBrace {
        //Err(format!("Expected `{{`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    let tmp = p_stmt(toks, pos);
    let mut stmt_node = ParseNode::new();
    match tmp {
        Ok((a,b)) => {stmt_node = a; pos = b;},
        Err(_) => {},
    }

    let tok = &toks[pos];
    if *tok != lexer::TokType::RBrace {
        //Err(format!("Expected `}}`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let mut fn_node = ParseNode::new();
    fn_node.entry = NodeType::Fn("main".to_string());
    fn_node.child.push(stmt_node);
    //println!("out function p_fn with pos = {}", pos);
    Ok((fn_node, pos))
}

fn p_stmt(
    toks: &Vec<lexer::TokType>,
    pos: usize,
) -> Result<(ParseNode, usize), String> {
    //println!("into function p_stmt");

    let tok = &toks[pos];
    if *tok != lexer::TokType::Kwd(lexer::KwdType::Ret) {
        //Err(format!("Expected 'return', found {:?} at {}", toks[pos], pos));
    }
    let mut pos = pos + 1;

    let tmp = p_exp(toks, pos);
    let mut exp_node = ParseNode::new();
    match tmp {
        Ok((a,b)) => {exp_node = a; pos = b;},
        Err(_) => {},
    }

    let tok = &toks[pos];
    if *tok != lexer::TokType::Semicolon {
        //Err(format!("Expected ';', found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let mut stmt_node = ParseNode::new();
    stmt_node.entry = NodeType::Stmt;
    stmt_node.child.push(exp_node);
    //println!("out function p_stmt with pos = {}, tok = {:?}", pos, toks[pos]);
    Ok((stmt_node, pos))
}

fn p_exp(
    toks: &Vec<lexer::TokType>,
    pos: usize,
) -> Result<(ParseNode, usize), String> {
    //println!("into function p_exp");
    let tok = &toks[pos];
    if *tok != lexer::TokType::Literal(2) {
        //Err(format!("Expected 'literal(2)`, found {:?} at {}", toks[pos], pos));
        panic!(format!("Expected 'literal(2)`, found {:?} at {}", toks[pos], pos))
    }
    let pos = pos + 1;

    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::Exp(2);
    //println!("out function p_exp with pos = {}, tok = {:?}", pos, toks[pos]);
    Ok((exp_node, pos))
}

pub fn parse_prog(input: &String, c_src_name: &String) -> Result<ParseNode, String> {
    let toks_res = lexer::lex(&input);
    let mut toks: std::vec::Vec<lexer::TokType> = Vec::new();

    match toks_res {
        Ok(n) => {toks = n},
        Err(_) => {/*Err(format!("Can not get lexemes"));*/},
    }
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
        NodeType::Prog(prog_name) => {
            format!("{}NodeType: Prog, Name:{} [\n{}\n{}]",idt_prefix, prog_name,
            print(tree.child.get(0).expect("parens need one child"), idt+1), idt_prefix)
        }
        NodeType::Fn(fn_name) => {
            format!("{}NodeType: Fn, Name: {} [\n{}\n{}]", idt_prefix, fn_name,
            print(tree.child.get(0).expect("no child"), idt+1), idt_prefix)
        }
        NodeType::Stmt => {
            format!("{}NodeType: Stmt, [\n{}\n{}]", idt_prefix,
            print(tree.child.get(0).expect("no child"), idt+1), idt_prefix)
        }
        NodeType::Exp(n) => {
            format!("{}NodeType: Exp, Value: {}", idt_prefix, n)
        }
    }
}
