use crate::lexer::TokType;
use crate::parser::{NodeType, ParseNode, StmtType};
use std::collections::{HashMap, HashSet};

// generate a std::String contains the assembly language code
static mut LABEL_COUNTER: i64 = -1;
fn gen_labels(prefix: String) -> String {
    unsafe {
        LABEL_COUNTER = LABEL_COUNTER + 1;
        return format!(".L{}{}", prefix, LABEL_COUNTER);
    }
}

static mut FLAG_FOR_MAIN_HAS_RET: bool = true;
fn fn_main_has_ret() {
    unsafe {
        FLAG_FOR_MAIN_HAS_RET = true;
    }
}

fn gen_fn_prologue(fn_name: String) -> String {
    let p = "        ";
    format!(
        "{}.global {}\n\
         {}.type {}, @function\n\
         {}:\n\
         {}:\n\
         {}.cfi_startproc\n\
         {}pushq	%rbp\n\
         {}.cfi_def_cfa_offset 16\n\
         {}.cfi_offset 6, -16\n\
         {}movq	%rsp, %rbp\n\
         {}.cfi_def_cfa_register 6\n\
         ",
        p,
        fn_name,
        p,
        fn_name,
        fn_name,
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
         {}.cfi_def_cfa 7, 8\n",
        p, p, p
    )
}
pub fn gen_prog(tree: &ParseNode) -> String {
    let p = "        ".to_string();

    // iter every function node
    let mut prog_body = String::new();
    let index_map: HashMap<String, isize> = HashMap::new();
    let idx: isize = 0;
    for it in tree.child.iter() {
        match &it.entry {
            NodeType::Fn(fn_name, _) => {
                let fn_prologue = gen_fn_prologue(fn_name.to_string());
                let fn_epilogue = gen_fn_epilogue();
                let fn_body = &gen_block(it, &index_map, idx, None, None, true);
                let tmp = unsafe {
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
                };
                let fn_tot = format!(
                    "{}\
                     {}\
                     {}\
                     {}\
                     {}.cfi_endproc\n\
                     {}:\n\
                     {}.size   {}, .-{}\n",
                    fn_prologue,
                    fn_body,
                    tmp,
                    fn_epilogue,
                    p,
                    gen_labels("FE".to_string()),
                    p,
                    fn_name,
                    fn_name
                );
                prog_body.push_str(&fn_tot);
            }
            _ => panic!("`{:?}` type should not be here", it.entry),
        }
    }
    match &tree.entry {
        NodeType::Prog(prog_name) => format!(
            "{}.file \"{}\"\n\
             {}\
             {}.ident	\"crust: 0.1 (By Haoran Wang)\"\n\
             {}.section	.note.GNU-stack,\"\",@progbits\n",
            p, prog_name, prog_body, p, p
        ),
        _ => panic!("Something went wrong in gen_prog"),
    }
}

pub fn gen_declare(
    tree: &ParseNode,
    index_map: &HashMap<String, isize>,
    scope: &HashSet<String>,
    idx: isize,
    lbb: &str,
    leb: &str,
    loop_in_label: Option<&str>,
    loop_out_label: Option<&str>,
) -> (HashMap<String, isize>, HashSet<String>, isize, String) {
    // println!("in gen_declare with {:?}", tree.entry);
    let p = "        ";
    let mut index_map = index_map.clone();
    let mut scope = scope.clone();
    let mut idx = idx;
    match &tree.entry {
        NodeType::Declare(var_name) => {
            if scope.contains(var_name) {
                panic!(
                    "Error: redeclaration of variable `{}` in the same scope",
                    var_name
                );
            } else {
                let tmp_str = format!("{}", var_name);
                scope.insert(tmp_str);
                // try to clear the previous index
                let tmp_str = format!("{}", var_name);
                index_map.insert(tmp_str, idx - 8);
                idx -= 8;
            }

            // judge whether it's initialized
            let mut e1 = String::new();

            if tree.child.is_empty() {
                // just declare, we initialized it with 0
                e1 = format!("        movq $0, %rax\n");
            } else {
                e1 = gen_stmt(
                    tree.child
                        .get(0)
                        .expect("Statement::Declare Node has no child"),
                    &index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                );
            }
            let s = format!(
                "{}\
                 {}pushq %rax\n",
                e1, p
            );
            (index_map, scope, idx, s)
        }
        _ => panic!("Type `{:?}` should not occur here", tree.entry),
    }
}

pub fn gen_for(tree: &ParseNode, index_map: &HashMap<String, isize>, idx: isize) -> String {
    let p = "        ".to_string();
    let label_begin_loop = gen_labels("BFOR".to_string());
    let label_end_loop = gen_labels("EFOR".to_string());

    let mut index_map = index_map.clone();
    let mut idx: isize = idx;
    // now in a new block now
    let mut scope: HashSet<String> = HashSet::new();
    match tree.entry {
        NodeType::Stmt(StmtType::ForDecl) => {
            let (index_map_new, scope_new, idx_new, init) = gen_declare(
                tree.child.get(0).unwrap(),
                &index_map,
                &scope,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
            );
            index_map = index_map_new.clone();
            idx = idx_new;
            scope = scope_new.clone();
            let condition = gen_stmt(
                tree.child.get(1).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
            );
            let post_exp = gen_stmt(
                tree.child.get(2).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
            );
            let stmt = gen_block(
                tree.child.get(3).unwrap(),
                &index_map,
                idx,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                true,
            );
            //           generate init (declare)
            // BEGN_LOOP:
            //           generate condition
            //           cmpq $0, %rax
            //           je  END_LOOP
            //           generate statement
            //           pos-expression
            //           jmp BEGIN_LOOP
            // END_LOOP:
            let b_deallocate = 8 * scope.len();
            format!(
                "{}\
                 {}:\n\
                 {}\
                 {}cmpq $0, %rax\n\
                 {}je {}\n\
                 {}\
                 {}\
                 {}jmp {}\n\
                 {}:\n\
                 {}addq ${}, %rsp\n",
                init,
                label_begin_loop,
                condition,
                p,
                p,
                label_end_loop,
                stmt,
                post_exp,
                p,
                label_begin_loop,
                label_end_loop,
                p,
                b_deallocate
            )
        }
        NodeType::Stmt(StmtType::For) => {
            let init = gen_stmt(
                tree.child.get(0).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
            );
            let condition = gen_stmt(
                tree.child.get(1).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
            );
            let post_exp = gen_stmt(
                tree.child.get(2).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
            );
            let stmt = gen_block(
                tree.child.get(3).unwrap(),
                &index_map,
                idx,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                true,
            );
            //           generate init
            // BEGN_LOOP:
            //           generate condition
            //           cmpq $0, %rax
            //           je  END_LOOP
            //           generate statement
            //           pos-expression
            //           jmp BEGIN_LOOP
            // END_LOOP:
            let b_deallocate = 8 * scope.len();
            format!(
                "{}\
                 {}:\n\
                 {}\
                 {}cmpq $0, %rax\n\
                 {}je {}\n\
                 {}\
                 {}\
                 {}jmp {}\n\
                 {}:\n\
                 {}addq ${}, %rsp\n",
                init,
                label_begin_loop,
                condition,
                p,
                p,
                label_end_loop,
                stmt,
                post_exp,
                p,
                label_begin_loop,
                label_end_loop,
                p,
                b_deallocate
            )
        }
        _ => panic!("Something wrong in gen_for"),
    }
}
/// gen_block() - into a new block, will have empty scope
pub fn gen_block(
    tree: &ParseNode,
    index_map: &HashMap<String, isize>,
    idx: isize,
    loop_in_label: Option<&str>,
    loop_out_label: Option<&str>,
    flag: bool,
) -> String {
    let p = "        ".to_string(); // 8 white spaces
    let label_begin_block = gen_labels("BB".to_string());
    let label_end_block = gen_labels("EB".to_string());
    // iter every block
    let mut stmts = String::new();
    let mut index_map = index_map.clone();
    let mut idx: isize = idx;
    let mut scope: HashSet<String> = HashSet::new();

    for it in &tree.child {
        // iter through every block-item
        match &it.entry {
            NodeType::Declare(_var_name) => {
                let (index_map_new, scope_new, idx_new, s) = gen_declare(
                    it,
                    &index_map,
                    &scope,
                    idx,
                    &label_begin_block,
                    &label_end_block,
                    loop_in_label,
                    loop_out_label,
                );
                index_map = index_map_new.clone();
                idx = idx_new;
                scope = scope_new.clone();
                stmts.push_str(&s);
            }
            NodeType::Stmt(StmtType::Compound) => {
                stmts.push_str(&gen_block(
                    it,
                    &index_map,
                    idx,
                    loop_in_label,
                    loop_out_label,
                    true,
                ));
            }
            _ => {
                let s = gen_stmt(
                    it,
                    &index_map,
                    idx,
                    &label_begin_block,
                    &label_end_block,
                    loop_in_label,
                    loop_out_label,
                );
                stmts.push_str(&s);
            }
        }
    }
    let b_deallocate = match flag {
        true => 8 * scope.len(),
        false => 0,
    };
    // let b_deallocate = 8 * scope.len(); // deallocate stack
    format!(
        "{}:\n\
         {}\
         {}:\n\
         {}addq ${}, %rsp\n",
        label_begin_block, stmts, label_end_block, p, b_deallocate
    )
}

pub fn gen_stmt(
    tree: &ParseNode,
    index_map: &HashMap<String, isize>,
    idx: isize,
    lbb: &str, // label_begin_block
    leb: &str, // label_end_block
    loop_in_label: Option<&str>,
    loop_out_label: Option<&str>,
) -> String {
    let p = "        ".to_string(); // 8 white spaces
    match &tree.entry {
        NodeType::ConditionalExp => {
            if tree.child.len() == 1 {
                // just one <logical-or-exp>
                gen_stmt(
                    tree.child
                        .get(0)
                        .expect("Conditional Expression has no child"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                )
            } else if tree.child.len() == 3 {
                // <logical-or-exp> "?" <exp> ":" <conditional-exp>
                let e1_as = gen_stmt(
                    tree.child.get(0).expect("Conditional expression no e1"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                );
                let e2_as = gen_stmt(
                    tree.child.get(1).expect("conditional expression no e2"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                );
                let e3_as = gen_stmt(
                    tree.child.get(2).expect("conditional expression no e3"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
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
        NodeType::Stmt(stmt) => match stmt {
            StmtType::Return => format!(
                "{}\
                 {}\
                 {}ret\n",
                gen_stmt(
                    tree.child.get(0).expect("Statement node no child"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                ),
                gen_fn_epilogue(),
                p
            ),
            StmtType::Conditional(_) => {
                let e1_as = gen_stmt(
                    tree.child.get(0).expect("Conditional node no e1"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                );
                let s1_as = gen_stmt(
                    tree.child.get(1).expect("conditional node no s1"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                );
                let s2_as: String = if tree.child.len() == 2 {
                    "".to_string()
                } else {
                    gen_stmt(
                        tree.child.get(2).expect("conditional node no s2"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
            StmtType::Exp => gen_stmt(
                tree.child.get(0).expect("Statement Node no child"),
                index_map,
                idx,
                lbb,
                leb,
                loop_in_label,
                loop_out_label,
            ),
            StmtType::Continue => match loop_in_label {
                Some(l) => format!("{}jmp {} # Continue\n", p, l),
                None => panic!("Continue should be in the loop scope"),
            },
            StmtType::Break => match loop_out_label {
                Some(l) => format!("{}jmp {} # Break\n", p, l),
                None => panic!("Break shoule be in the loop scope"),
            },
            StmtType::For | StmtType::ForDecl => gen_for(tree, index_map, idx),
            StmtType::Do => {
                // LBB.
                // stmt
                // exp
                // cmpq $1, %rax
                // je  LBB
                // LEB
                let lbb = gen_labels("BDO".to_string());
                let leb = gen_labels("EDO".to_string());
                let stmts = gen_block(
                    tree.child.get(0).unwrap(),
                    index_map,
                    idx,
                    loop_in_label,
                    loop_out_label,
                    true,
                ); // should enter a new scope
                let exp = gen_stmt(
                    tree.child.get(1).unwrap(),
                    index_map,
                    idx,
                    &lbb,
                    &leb,
                    Some(&lbb),
                    Some(&leb),
                );
                format!(
                    "{}:\n\
                     {}\
                     {}\
                     {}cmpq $1, %rax\n\
                     {}je   {}\n\
                     {}:\n",
                    lbb, stmts, exp, p, p, lbb, leb
                )
            }
            StmtType::While => {
                // LBB.
                // exp
                // cmpq $1, %rax
                // jne LEB
                // stmt
                // jmp LBB
                // LEB.
                let lbb = gen_labels("BWHILE".to_string());
                let leb = gen_labels("EWHILE".to_string());

                let exp = gen_stmt(
                    tree.child.get(0).unwrap(),
                    index_map,
                    idx,
                    &lbb,
                    &leb,
                    Some(&lbb),
                    Some(&leb),
                );
                let stmts = gen_block(
                    tree.child.get(1).unwrap(),
                    index_map,
                    idx,
                    Some(&lbb),
                    Some(&leb),
                    true,
                ); // should enter a new scope
                format!(
                    "{}:\n\
                     {}\
                     {}cmpq $1, %rax\n\
                     {}jne {}\n\
                     {}\
                     {}jmp {}\n\
                     {}:\n",
                    lbb, exp, p, p, leb, stmts, p, lbb, leb
                )
            }
            StmtType::Compound => {
                gen_block(tree, index_map, idx, loop_in_label, loop_out_label, true)
            }
        },
        NodeType::AssignNode(var_name) => {
            match index_map.get(var_name) {
                Some(t) => {
                    // declared before, that's ok
                    let e1 = gen_stmt(
                        tree.child
                            .get(0)
                            .expect("Statement::Declare Node has no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    );
                    let get_result = index_map.get(var_name);
                    let mut va_offset: isize = -8;
                    match get_result {
                        Some(t) => {
                            va_offset = *t;
                        }
                        None => panic!("Something went wrong in gen::gen_stmt()"),
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
        NodeType::UnExp(Op) => match Op {
            TokType::Minus => format!(
                "{}\
                 {}neg %rax\n",
                gen_stmt(
                    tree.child.get(0).expect("UnExp<-> no child"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                ),
                p
            ),
            TokType::Tilde => format!(
                "{}\
                 {}not %rax\n",
                gen_stmt(
                    tree.child.get(0).expect("UnExp<~> no child"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                ),
                p
            ),
            TokType::Exclamation => format!(
                "{}\
                 {}cmp  $0, %rax\n\
                 {}movq $0, %rax\n\
                 {}sete %al\n",
                gen_stmt(
                    tree.child.get(0).expect("UnExp<!> node no child"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
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
        },
        NodeType::BinExp(Op) => {
            match Op {
                TokType::Plus => format!(
                    "{}\
                     {}pushq %rax\n\
                     {}\
                     {}popq %rcx\n\
                     {}addq %rcx, %rax\n",
                    gen_stmt(
                        tree.child.get(0).expect("BinExp has no lhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp has no rhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(1).expect("BinExp has no rhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(0).expect("BinExp has no lhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp has no lhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp has no rhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(1).expect("BinExp has no rhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(0).expect("BinExp has no lhs"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                        gen_stmt(
                            tree.child.get(0).expect("BinExp<||> node no child"),
                            index_map,
                            idx,
                            lbb,
                            leb,
                            loop_in_label,
                            loop_out_label,
                        ),
                        p,
                        p,
                        clause2_label,
                        p,
                        p,
                        end_label,
                        clause2_label,
                        gen_stmt(
                            tree.child.get(1).expect("BinExp<||> node no child"),
                            index_map,
                            idx,
                            lbb,
                            leb,
                            loop_in_label,
                            loop_out_label,
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
                        gen_stmt(
                            tree.child.get(0).expect("BinExp<||> node no child"),
                            index_map,
                            idx,
                            lbb,
                            leb,
                            loop_in_label,
                            loop_out_label,
                        ),
                        p,
                        p,
                        clause2_label,
                        p,
                        end_label,
                        clause2_label,
                        gen_stmt(
                            tree.child.get(1).expect("BinExp<||> node no child"),
                            index_map,
                            idx,
                            lbb,
                            leb,
                            loop_in_label,
                            loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
                    gen_stmt(
                        tree.child.get(0).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
                    ),
                    p,
                    gen_stmt(
                        tree.child.get(1).expect("BinExp<==> node no child"),
                        index_map,
                        idx,
                        lbb,
                        leb,
                        loop_in_label,
                        loop_out_label,
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
        NodeType::ExpOption => {
            if tree.child.len() == 1 {
                gen_stmt(
                    tree.child
                        .get(0)
                        .expect(&format!("{:?} node no child", &tree.entry)),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                )
            } else {
                // null exp
                // movq 1, %rax
                format!("{}movq $1, %rax\n", p)
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
        | NodeType::LogicalAndExp => gen_stmt(
            tree.child
                .get(0)
                .expect(&format!("{:?} node no child", &tree.entry)),
            index_map,
            idx,
            lbb,
            leb,
            loop_in_label,
            loop_out_label,
        ),
        _ => panic!(format!(
            "Node `{:?}` not implemented in gen::gen_stmt()\n",
            &tree.entry
        )),
    }
}
