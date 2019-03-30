use crate::parser::{NodeType, ParseNode};

// generate a std::String contains the assembly language code
/*

*/
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
            "{}movl ${},  %eax\n{}ret\n",
            idt_prefix,
            gen_as(tree.child.get(0).expect("Statement node no child")),
            idt_prefix
        ),
        NodeType::Exp(n) => format!("{}", n),
    }
}
