use crate::lexer::TokType;
use crate::parser::{stmt_type, NodeType, ParseNode};
use std::collections::HashMap;

// generate a std::String contains the assembly language code
static mut LABEL_COUNTER: i64 = -1;
fn gen_labels(prefix: String) -> String {
    // XXX: should improve it, now just produce some thing like `_label.1` `_label.2`
    unsafe {
        LABEL_COUNTER = LABEL_COUNTER + 1;
        return format!(".L{}{}", prefix, LABEL_COUNTER);
    }
}

fn gen_fn_prologue() -> String {
    let p = "        ";
    format!(
        "{}:\n\
         {}.cfi_startproc\n\
         {}pushl	%ebp\n\
         {}.cfi_def_cfa_offset 8\n\
         {}.cfi_offset 5, -8\n\
         {}movl	%esp, %ebp\n\
         {}.cfi_def_cfa_register 5\n\
         {}call	__x86.get_pc_thunk.ax\n\
         {}addl	$_GLOBAL_OFFSET_TABLE_, %eax\n\
         ",
        gen_labels("FB".to_string()),
        p,
        p,
        p,
        p,
        p,
        p,
        p,
        p
    )
}

fn gen_fn_epilogue() -> String {
    let p = "        ";
    format!(
        "\
         {}popl	%ebp\n\
         {}.cfi_restore 5\n\
         {}.cfi_def_cfa 4, 4\n\
         ",
        p, p, p
    )
}
pub fn gen_as(tree: &ParseNode) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::Prog(prog_name) => format!(
            ".code32\n\
             {}.file \"{}\"\n\
             {}\
	           {}.section	.text.__x86.get_pc_thunk.ax,\"axG\",@progbits,__x86.get_pc_thunk.ax,comdat\n\
	           {}.globl	__x86.get_pc_thunk.ax\n\
	           {}.hidden	__x86.get_pc_thunk.ax\n\
	           {}.type	__x86.get_pc_thunk.ax, @function\n\
             __x86.get_pc_thunk.ax:\n\
	           {}.cfi_startproc\n\
	           {}movl	(%esp), %eax\n\
	           {}ret\n\
	           {}.cfi_endproc\n\
	           {}.ident	\"crust: 0.1 (By Haoran Wang)\"\n\
	           {}.section	.note.GNU-stack,\"\",@progbits\n\
             ",
            p, prog_name,
            gen_as(tree.child.get(0).expect("Program node no child")),
            p,p,p,p,p,p,p,p,p,p
        ),
        NodeType::Fn(fn_name, _) => {
            let mut stmts = String::new();
            for it in &tree.child {
                stmts.push_str(&gen_as(&it));
            }
            format!(
                "{}.global {}\n\
                 {}.type {}, @function\n\
                 {}:\n\
                 {}\
                 {}\
                 {}.cfi_endproc\n\
                 {}:\n\
                 {}.size	{}, .-{}\n\
                 ",
                p,
                fn_name,
                p,
                fn_name,
                fn_name,
                gen_fn_prologue(),
                stmts,
                p,
                gen_labels("FE".to_string()),
                p,
                fn_name,
                fn_name
            )
        }
        NodeType::Stmt(stmt_type::Return) => format!(
            "{}\
             {}\
             {}ret\n",
            gen_as(tree.child.get(0).expect("Statement node no child")),
            gen_fn_epilogue(),
            p
        ),
        NodeType::UnExp(Op) => gen_unexp(tree, Op),
        NodeType::BinExp(Op) => gen_binexp(tree, Op),
        NodeType::Const(n) => format!("{}movl ${}, %eax\n", p, n),
        NodeType::EqualityExp
        | NodeType::RelationalExp
        | NodeType::Term
        | NodeType::Exp
        | NodeType::Factor
        | NodeType::AdditiveExp
        | NodeType::LogicalOrExp
        | NodeType::LogicalAndExp => gen_as(
            tree.child
                .get(0)
                .expect(&format!("{:?} node no child", &tree.entry)),
        ),
        _ => panic!(format!(
            "Node `{:?}` not implemented in gen::gen_as()\n",
            &tree.entry
        )),
    }
}

fn gen_unexp(tree: &ParseNode, Op: &TokType) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match Op {
        TokType::Minus => format!(
            "{}\
             {}neg %eax\n",
            gen_as(tree.child.get(0).expect("UnExp<-> no child")),
            p
        ),
        TokType::Tilde => format!(
            "{}\
             {}not %eax\n",
            gen_as(tree.child.get(0).expect("UnExp<~> no child")),
            p
        ),
        TokType::Exclamation => format!(
            "{}\
             {}cmp  $0, %eax\n\
             {}movl $0, %eax\n\
             {}sete %al\n",
            gen_as(tree.child.get(0).expect("UnExp<!> node no child")),
            p,
            p,
            p
        ),
        TokType::Lt => format!("Error: `<` not implemented"),
        TokType::Gt => format!("Error: `>` not implemented"),
        _ => panic!(format!(
            "Unary Operator `{:?}` not implemented in gen::gen_unexp()\n",
            Op
        )),
    }
}

fn gen_binexp(tree: &ParseNode, Op: &TokType) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match Op {
        TokType::Plus => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}addl %ecx, %eax\n",
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            p,
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            p,
            p
        ),
        TokType::Minus => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}subl %ecx, %eax\n", // subl src, dst : dst - src -> dst
            //   let %eax = dst = e1, %ecx = src = e2
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            p,
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            p,
            p
        ),
        TokType::Multi => format!(
            "{}\
             {}pushl %eax\n\
             {}\
             {}popl %ecx\n\
             {}imul %ecx, %eax\n",
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            p,
            gen_as(tree.child.get(1).expect("BinExp has no rhs")),
            p,
            p
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
            p,
            gen_as(tree.child.get(0).expect("BinExp has no lhs")),
            p,
            p,
            p
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
            p,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            p,
            p,
            p,
            p
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
            p,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            p,
            p,
            p,
            p
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
            p,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            p,
            p,
            p,
            p
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
            p,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            p,
            p,
            p,
            p
        ),
        TokType::Or => {
            let clause2_label = gen_labels(format!("CLAUSE"));
            let end_label = gen_labels(format!("END"));
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
                p,
                p,
                clause2_label,
                p,
                p,
                end_label,
                clause2_label,
                gen_as(tree.child.get(1).expect("BinExp<||> node no child")),
                p,
                p,
                p,
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
                p,
                p,
                clause2_label,
                p,
                end_label,
                clause2_label,
                gen_as(tree.child.get(1).expect("BinExp<||> node no child")),
                p,
                p,
                p,
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
            p,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            p,
            p,
            p,
            p
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
            p,
            gen_as(tree.child.get(1).expect("BinExp<==> node no child")),
            p,
            p,
            p,
            p
        ),
        _ => panic!(format!(
            "Error: Binary Operator `{:?}` not implemented in gen::gen_binexp()\n",
            Op
        )),
    }
}
