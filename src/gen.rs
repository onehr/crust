use crate::lexer::TokType;
use crate::parser::{NodeType, ParseNode};

// generate a std::String contains the assembly language code

pub fn gen_as(tree: &ParseNode) -> String {
    let idt_prefix = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::Prog(prog_name) => format!(
            ".code32\n{}.file \"{}\"\n{}",
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
        NodeType::UnExp(Op) => gen_unexp(tree, Op),
        NodeType::BinExp(Op) => gen_binexp(tree, Op),
        NodeType::Const(n) => format!("{}movl ${}, %eax\n", idt_prefix, n),
        NodeType::EqualityExp
        | NodeType::RelationalExp
        | NodeType::Term
        | NodeType::Exp
        | NodeType::Factor
        | NodeType::AdditiveExp
        | NodeType::LogicalAndExp => gen_as(
            tree.child
                .get(0)
                .expect(&format!("{:?} node no child", &tree.entry)),
        ),
    }
}

fn gen_unexp(tree: &ParseNode, Op: &TokType) -> String {
    let idt_prefix = "        ".to_string(); // 8 white spaces
    match Op {
        TokType::Minus => format!(
            "{}\
             {}neg %eax\n",
            gen_as(tree.child.get(0).expect("UnExp<-> no child")),
            idt_prefix
        ),
        TokType::Tilde => format!(
            "{}\
             {}not %eax\n",
            gen_as(tree.child.get(0).expect("UnExp<~> no child")),
            idt_prefix
        ),
        TokType::Exclamation => format!(
            "{}\
             {}cmp  $0, %eax\n\
             {}movl $0, %eax\n\
             {}sete %al\n",
            gen_as(tree.child.get(0).expect("UnExp<!> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::Lt => format!("Error: `<` not implemented"),
        TokType::Gt => format!("Error: `>` not implemented"),
        _ => panic!(format!(
            "Unary Operator `{:?}` not implemented in gen::gen_as()\n",
            Op
        )),
    }
}
static mut LABEL_COUNTER: i64 = -1;

fn gen_labels(prefix: String) -> String {
    // XXX: should improve it, now just produce some thing like `_label.1` `_label.2`
    unsafe {
        LABEL_COUNTER = LABEL_COUNTER + 1;
        return format!(".LBF_{}.{}", prefix, LABEL_COUNTER);
    }
}

fn gen_binexp(tree: &ParseNode, Op: &TokType) -> String {
    let idt_prefix = "        ".to_string(); // 8 white spaces
    match Op {
        TokType::Plus => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}addl %ecx, %eax\n",
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            idt_prefix,
            idt_prefix
        ),
        TokType::Minus => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}subl %ecx, %eax\n", // subl src, dst : dst - src -> dst
            //   let %eax = dst = e1, %ecx = src = e2
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            idt_prefix,
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            idt_prefix,
            idt_prefix
        ),
        TokType::Multi => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}imul %ecx, %eax\n",
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            idt_prefix,
            idt_prefix
        ),
        TokType::Splash => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}xorl %edx, %edx\n\
             {}idivl %ecx\n",
            // let eax = e1, edx = 0, ecx = e2
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            idt_prefix,
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::Equal => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}cmpl %eax, %ecx # set ZF on if %eax == %ecx, set it off otherwise\n\
             {}movl $0, %eax   # zero out EAX, does not change flag\n\
             {}sete %al\n",
            gen_as(tree.child.get(0).expect("BinExp<==> node no child")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::NotEqual => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}cmpl %eax, %ecx # set ZF on if %eax == %ecx, set it off otherwise\n\
             {}movl $0, %eax   # zero out EAX, does not change flag\n\
             {}setne %al\n",
            gen_as(tree.child.get(0).expect("BinExp<==> node no child")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::LessEqual => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}cmpl %eax, %ecx # set ZF on if %eax == %ecx, set it off otherwise\n\
             {}movl $0, %eax   # zero out EAX, does not change flag\n\
             {}setle %al\n",
            gen_as(tree.child.get(0).expect("BinExp<==> node no child")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::GreaterEqual => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}cmpl %eax, %ecx # set ZF on if %eax == %ecx, set it off otherwise\n\
             {}movl $0, %eax   # zero out EAX, does not change flag\n\
             {}setge %al\n",
            gen_as(tree.child.get(0).expect("BinExp<==> node no child")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::Or => {
            let clause2_label = gen_labels(format!("clause"));
            let end_label = gen_labels(format!("end"));
            format!(
                "{}\
                 {}cmpl $0, %eax\n\
                 {}je {}\n\
                 {}movl $1, %eax\n\
                 {}jmp {}\n\
                 {}:\n\
                 {}\
                 {}cmpl $0, %eax\n\
                 {}movl $0, %eax\n\
                 {}setne %al\n\
                 {}: # end of clause here\n",
                gen_as(tree.child.get(0).expect("BinExp<||> node no child")),
                idt_prefix,
                idt_prefix,
                clause2_label,
                idt_prefix,
                idt_prefix,
                end_label,
                clause2_label,
                gen_as(tree.child.get(1).expect("BinExp<||> node no child")),
                idt_prefix,
                idt_prefix,
                idt_prefix,
                end_label
            )
        }
        TokType::And => {
            let clause2_label = gen_labels(format!("clause"));
            let end_label = gen_labels(format!("end"));
            format!(
                "{}\
                 {}cmpl $0, %eax\n\
                 {}jne {}\n\
                 {}jmp {}\n\
                 {}:\n\
                 {}\
                 {}cmpl $0, %eax\n\
                 {}movl $0, %eax\n\
                 {}setne %al\n\
                 {}: # end of clause here\n",
                gen_as(tree.child.get(0).expect("BinExp<||> node no child")),
                idt_prefix,
                idt_prefix,
                clause2_label,
                idt_prefix,
                end_label,
                clause2_label,
                gen_as(tree.child.get(1).expect("BinExp<||> node no child")),
                idt_prefix,
                idt_prefix,
                idt_prefix,
                end_label
            )
        }
        TokType::Lt => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}cmpl %eax, %ecx # set ZF on if %eax == %ecx, set it off otherwise\n\
             {}movl $0, %eax   # zero out EAX, does not change flag\n\
             {}setl %al\n",
            gen_as(tree.child.get(0).expect("BinExp<==> node no child")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        TokType::Gt => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}cmpl %eax, %ecx # set ZF on if %eax == %ecx, set it off otherwise\n\
             {}movl $0, %eax   # zero out EAX, does not change flag\n\
             {}setg %al\n",
            gen_as(tree.child.get(0).expect("BinExp<==> node no child")),
            idt_prefix,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            idt_prefix,
            idt_prefix,
            idt_prefix,
            idt_prefix
        ),
        _ => panic!(format!(
            "Error: Binary Operator `{:?}` not implemented in gen::gen_binexp()\n",
            Op
        )),
    }
}
