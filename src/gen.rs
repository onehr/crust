use crate::lexer::TokType;
use crate::parser::{stmt_type, NodeType, ParseNode};
use std::collections::HashMap;

// generate a std::String contains the assembly language code
static mut LABEL_COUNTER: i64 = -1;
fn gen_labels(prefix: String) -> String {
    unsafe {
        LABEL_COUNTER = LABEL_COUNTER + 1;
        return format!(".L{}{}", prefix, LABEL_COUNTER);
    }
}

static mut FLAG_FOR_MAIN_HAS_RET: bool = false;
fn fn_main_has_ret() {
    unsafe {
        FLAG_FOR_MAIN_HAS_RET = true;
    }
}

fn gen_fn_prologue() -> String {
    let p = "        ";
    format!(
        "{}:\n\
         {}.cfi_startproc\n\
         {}pushq	%rbp\n\
         {}.cfi_def_cfa_offset 16\n\
         {}.cfi_offset 6, -16\n\
         {}movq	%rsp, %rbp\n\
         {}.cfi_def_cfa_register 6\n\
         ",
        gen_labels("FB".to_string()),
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
        "{}movq %rbp, %rsp\n\
         {}popq	%rbp\n\
         {}.cfi_def_cfa 7, 8\n\
         ",
        p, p, p
    )
}
pub fn gen_prog(tree: &ParseNode) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::Prog(prog_name) => format!(
            "{}.file \"{}\"\n\
             {}\
             {}.ident	\"crust: 0.1 (By Haoran Wang)\"\n\
             {}.section	.note.GNU-stack,\"\",@progbits\n\
             ",
            p,
            prog_name,
            gen_fn(tree.child.get(0).expect("Program node no child")),
            p,
            p
        ),
        _ => panic!("Not a program"),
    }
}
pub fn gen_fn(tree: &ParseNode) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::Fn(fn_name, _) => {
            // if this function is main,
            // evan without a return val, we should add return 0 before leave main function
            let mut stmts = String::new();
            let index_map: &mut HashMap<String, isize> = &mut HashMap::new();
            let idx: &mut isize = &mut -8;
            for it in &tree.child {
                if fn_name == "main" && it.entry == NodeType::Stmt(stmt_type::Return) {
                    fn_main_has_ret();
                }
                stmts.push_str(&gen_as(&it, index_map, idx));
            }
            format!(
                "{}.global {}\n\
                 {}.type {}, @function\n\
                 {}:\n\
                 {}\
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
                unsafe {
                    if FLAG_FOR_MAIN_HAS_RET == false {
                        format!(
                            "{}movq $0, %rax\n\
                             {}\
                             {}ret\n",
                            p,
                            gen_fn_epilogue(),
                            p
                        )
                    } else {
                        "".to_string()
                    }
                },
                p,
                gen_labels("FE".to_string()),
                p,
                fn_name,
                fn_name
            )
        }
        _ => panic!("Not a function node"),
    }
}
pub fn gen_as(tree: &ParseNode, index_map: &mut HashMap<String, isize>, idx: &mut isize) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::ConditionalExp => {
            if tree.child.len() == 1 {
                // just one <logical-or-exp>
                gen_as(
                    tree.child
                        .get(0)
                        .expect("Conditional Expression has no child"),
                    index_map,
                    idx,
                )
            } else if tree.child.len() == 3 {
                // <logical-or-exp> "?" <exp> ":" <conditional-exp>
                let e1_as = gen_as(
                    tree.child.get(0).expect("Conditional expression no e1"),
                    index_map,
                    idx,
                );
                let e2_as = gen_as(
                    tree.child.get(1).expect("conditional expression no e2"),
                    index_map,
                    idx,
                );
                let e3_as = gen_as(
                    tree.child.get(2).expect("conditional expression no e3"),
                    index_map,
                    idx,
                );

                let label_e3 = gen_labels(format!("E3"));
                let label_end = gen_labels(format!("ENDCOND"));
                format!(
                    "{}\
                     {}cmpq $0, %rax\n\
                     {}je {}\n\
                     {}\
                     {}jmp {}\n\
                     {}:\n\
                     {}\
                     {}:\n",
                    e1_as, p, p, label_e3, e2_as, p, label_end, label_e3, e3_as, label_end,
                )
            } else {
                panic!("Error: something wrong in conditional expression")
            }
        }
        NodeType::Declare(var_name) => {
            match index_map.get(var_name) {
                Some(_) => panic!("Error: redeclaration of variable `{}`", var_name),
                None => {
                    // Ok;
                    let tmp_str = format!("{}", var_name);
                    index_map.insert(tmp_str, *idx);
                    *idx -= 8;
                }
            }
            // judge whether it's initialized
            let mut e1 = String::new();

            if tree.child.is_empty() {
                // just declare, we initialized it with 0
                e1 = format!("{}movq $0, %rax\n", p);
            } else {
                e1 = gen_as(
                    tree.child
                        .get(0)
                        .expect("Statement::Declare Node has no child"),
                    index_map,
                    idx,
                )
            }
            format!(
                "{}\
                 {}pushq %rax\n",
                e1, p
            )
        }
        NodeType::Stmt(stmt) => match stmt {
            stmt_type::Return => format!(
                "{}\
                 {}\
                 {}ret\n",
                gen_as(
                    tree.child.get(0).expect("Statement node no child"),
                    index_map,
                    idx
                ),
                gen_fn_epilogue(),
                p
            ),
            stmt_type::Conditional(_) => {
                let e1_as = gen_as(
                    tree.child.get(0).expect("Conditional node no e1"),
                    index_map,
                    idx,
                );
                let s1_as = gen_as(
                    tree.child.get(1).expect("conditional node no s1"),
                    index_map,
                    idx,
                );
                let s2_as: String = if tree.child.len() == 2 {
                    "".to_string()
                } else {
                    gen_as(
                        tree.child.get(2).expect("conditional node no s2"),
                        index_map,
                        idx,
                    )
                };
                let label_s2 = gen_labels(format!("S2"));
                let label_end = gen_labels(format!("ENDIF"));
                format!(
                    "{}\
                     {}cmpq $0, %rax\n\
                     {}je {}\n\
                     {}\
                     {}jmp {}\n\
                     {}:\n\
                     {}\
                     {}:\n",
                    e1_as, p, p, label_s2, s1_as, p, label_end, label_s2, s2_as, label_end,
                )
            }
            stmt_type::Exp => gen_as(
                tree.child.get(0).expect("Statement Node no child"),
                index_map,
                idx,
            ),
        },
        NodeType::AssignNode(var_name) => {
            match index_map.get(var_name) {
                Some(t) => {
                    // declared before, that's ok
                    let e1 = gen_as(
                        tree.child
                            .get(0)
                            .expect("Statement::Declare Node has no child"),
                        index_map,
                        idx,
                    );
                    let get_result = index_map.get(var_name);
                    let mut va_offset: isize = -8;
                    match get_result {
                        Some(t) => {
                            va_offset = *t;
                        }
                        None => panic!("Something went wrong in gen::gen_as()"),
                    }
                    format!(
                        "{}\
                         {}movq %rax, {}(%rbp)\n",
                        e1, p, va_offset
                    )
                }
                None => {
                    // Not declared before, that's not ok
                    panic!("Error: Use un-declared variable `{}`", var_name)
                }
            }
        }
        NodeType::UnExp(Op) => gen_unexp(tree, Op, index_map, idx),
        NodeType::BinExp(Op) => gen_binexp(tree, Op, index_map, idx),
        NodeType::Const(n) => format!("{}movq ${}, %rax\n", p, n),
        NodeType::Var(var_name) => {
            let var_offset = index_map.get(var_name);
            match var_offset {
                Some(t) => {
                    let var_offset = t;
                    format!("{}movq {}(%rbp), %rax\n", p, var_offset)
                }
                None => panic!(format!("Use of undeclared variable `{}`", var_name)),
            }
        }
        NodeType::EqualityExp
        | NodeType::RelationalExp
        | NodeType::Term
        | NodeType::Exp
        | NodeType::Factor
        | NodeType::AdditiveExp
        | NodeType::LogicalOrExp
        | NodeType::Block
        | NodeType::LogicalAndExp => gen_as(
            tree.child
                .get(0)
                .expect(&format!("{:?} node no child", &tree.entry)),
            index_map,
            idx,
        ),
        _ => panic!(format!(
            "Node `{:?}` not implemented in gen::gen_as()\n",
            &tree.entry
        )),
    }
}

fn gen_unexp(
    tree: &ParseNode,
    Op: &TokType,
    index_map: &mut HashMap<String, isize>,
    idx: &mut isize,
) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match Op {
        TokType::Minus => format!(
            "{}\
             {}neg %rax\n",
            gen_as(
                tree.child.get(0).expect("UnExp<-> no child"),
                index_map,
                idx
            ),
            p
        ),
        TokType::Tilde => format!(
            "{}\
             {}not %rax\n",
            gen_as(
                tree.child.get(0).expect("UnExp<~> no child"),
                index_map,
                idx
            ),
            p
        ),
        TokType::Exclamation => format!(
            "{}\
             {}cmp  $0, %rax\n\
             {}movq $0, %rax\n\
             {}sete %al\n",
            gen_as(
                tree.child.get(0).expect("UnExp<!> node no child"),
                index_map,
                idx
            ),
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

fn gen_binexp(
    tree: &ParseNode,
    Op: &TokType,
    index_map: &mut HashMap<String, isize>,
    idx: &mut isize,
) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match Op {
        TokType::Plus => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}addq %rcx, %rax\n",
            gen_as(
                tree.child.get(0).expect("BinExp has no lhs"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp has no rhs"),
                index_map,
                idx
            ),
            p,
            p
        ),
        TokType::Minus => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}subq %rcx, %rax\n", // subl src, dst : dst - src -> dst
            //   let %rax = dst = e1, %rcx = src = e2
            gen_as(
                tree.child.get(1).expect("BinExp has no rhs"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(0).expect("BinExp has no lhs"),
                index_map,
                idx
            ),
            p,
            p
        ),
        TokType::Multi => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}imul %rcx, %rax\n",
            gen_as(
                tree.child.get(0).expect("BinExp has no lhs"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp has no rhs"),
                index_map,
                idx
            ),
            p,
            p
        ),
        TokType::Splash => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}xorq %rdx, %rdx\n\
             {}idivq %rcx\n",
            // let eax = e1, edx = 0, ecx = e2
            gen_as(
                tree.child.get(1).expect("BinExp has no rhs"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(0).expect("BinExp has no lhs"),
                index_map,
                idx
            ),
            p,
            p,
            p
        ),
        TokType::Equal => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise\n\
             {}movq $0, %rax   # zero out EAX, does not change flag\n\
             {}sete %al\n",
            gen_as(
                tree.child.get(0).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            p,
            p,
            p
        ),
        TokType::NotEqual => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise\n\
             {}movq $0, %rax   # zero out EAX, does not change flag\n\
             {}setne %al\n",
            gen_as(
                tree.child.get(0).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            p,
            p,
            p
        ),
        TokType::LessEqual => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise\n\
             {}movq $0, %rax   # zero out EAX, does not change flag\n\
             {}setle %al\n",
            gen_as(
                tree.child.get(0).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            p,
            p,
            p
        ),
        TokType::GreaterEqual => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise\n\
             {}movq $0, %rax   # zero out EAX, does not change flag\n\
             {}setge %al\n",
            gen_as(
                tree.child.get(0).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
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
                 {}cmpq $0, %rax\n\
                 {}je {}\n\
                 {}movq $1, %rax\n\
                 {}jmp {}\n\
                 {}:\n\
                 {}\
                 {}cmpq $0, %rax\n\
                 {}movq $0, %rax\n\
                 {}setne %al\n\
                 {}: # end of clause here\n",
                gen_as(
                    tree.child.get(0).expect("BinExp<||> node no child"),
                    index_map,
                    idx
                ),
                p,
                p,
                clause2_label,
                p,
                p,
                end_label,
                clause2_label,
                gen_as(
                    tree.child.get(1).expect("BinExp<||> node no child"),
                    index_map,
                    idx
                ),
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
                 {}cmpq $0, %rax\n\
                 {}jne {}\n\
                 {}jmp {}\n\
                 {}:\n\
                 {}\
                 {}cmpq $0, %rax\n\
                 {}movq $0, %rax\n\
                 {}setne %al\n\
                 {}: # end of clause here\n",
                gen_as(
                    tree.child.get(0).expect("BinExp<||> node no child"),
                    index_map,
                    idx
                ),
                p,
                p,
                clause2_label,
                p,
                end_label,
                clause2_label,
                gen_as(
                    tree.child.get(1).expect("BinExp<||> node no child"),
                    index_map,
                    idx
                ),
                p,
                p,
                p,
                end_label
            )
        }
        TokType::Lt => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise\n\
             {}movq $0, %rax   # zero out EAX, does not change flag\n\
             {}setl %al\n",
            gen_as(
                tree.child.get(0).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            p,
            p,
            p
        ),
        TokType::Gt => format!(
            "{}\
             {}pushq %rax\n\
             {}\
             {}popq %rcx\n\
             {}cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise\n\
             {}movq $0, %rax   # zero out EAX, does not change flag\n\
             {}setg %al\n",
            gen_as(
                tree.child.get(0).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
            p,
            gen_as(
                tree.child.get(1).expect("BinExp<==> node no child"),
                index_map,
                idx
            ),
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
