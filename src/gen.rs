use crate::lexer;
use crate::parser::{NodeType, ParseNode};

// generate a std::String contains the assembly language code

pub fn gen_as(tree: &ParseNode) -> String {
    let idt_prefix = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::Prog(prog_name) => format!(
            "{}.file \"{}\"\n{}",
            idt_prefix,
            prog_name,
            gen_as(tree.child.get(0).expect("Program node no child"))
        ),
        NodeType::Fn(fn_name) => format!(
            "{}.global {}\n{}.type {}, @function\n{}:\n{}",
            idt_prefix,
            fn_name,
            idt_prefix,
            fn_name,
            fn_name,
            gen_as(tree.child.get(0).expect("Function node no child"))
        ),
        NodeType::Stmt => format!(
            "{}\
             {}ret\n",
            gen_as(tree.child.get(0).expect("Statement node no child")),
            idt_prefix
        ),
        NodeType::UnExp(t) => match t {
            lexer::TokType::Minus => format!(
                "{}\
                 {}neg %eax\n",
                gen_as(tree.child.get(0).expect("UnExp<-> no child")),
                idt_prefix
            ),
            lexer::TokType::Tilde => format!(
                "{}\
                 {}not %eax\n",
                gen_as(tree.child.get(0).expect("UnExp<~> no child")),
                idt_prefix
            ),
            lexer::TokType::Exclamation => format!(
                "{}\
                 {}cmp  $0, %eax\n\
                 {}movl $0, %eax\n\
                 {}sete %al\n",
                gen_as(tree.child.get(0).expect("UnExp<!> node no child")),
                idt_prefix,
                idt_prefix,
                idt_prefix
            ),
            _ => format!(""),
        },
        NodeType::Exp => gen_as(tree.child.get(0).expect("Expression node no child")),
        NodeType::Const(n) => format!("{}movl ${}, %eax\n", idt_prefix, n),
    }
}
