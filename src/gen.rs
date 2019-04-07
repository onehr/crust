use crate::lexer::TokType;
use crate::parser::{NodeType, ParseNode, StmtType, DataType};
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
        "{}.text\n\
         {}.global {}\n\
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

fn compute_const(tree: &ParseNode) -> i64 {
    match &tree.entry {
        NodeType::BinExp(op) => {
            let lhs = compute_const(tree.child.get(0).unwrap());
            let rhs = compute_const(tree.child.get(1).unwrap());
            match op {
                TokType::Plus => {
                    return lhs + rhs;
                }
                TokType::Multi => {
                    return lhs * rhs;
                }
                TokType::Splash => {
                    return lhs / rhs;
                }
                TokType::And => {
                    if lhs != 0 && rhs != 0 {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::Or => {
                    if lhs != 0 || rhs != 0 {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::Equal => {
                    if lhs == rhs {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::NotEqual => {
                    if lhs != rhs {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::LessEqual => {
                    if lhs <= rhs {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::GreaterEqual => {
                    if lhs >= rhs {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::Lt => {
                    if lhs < rhs {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                TokType::Gt => {
                    if lhs > rhs {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                _ => panic!("{:?} should not occur in global variable initialization"),
            }
        }
        NodeType::UnExp(op) => {
            let child_val = compute_const(tree.child.get(0).unwrap());
            match op {
                TokType::Minus => {
                    return -child_val;
                }
                TokType::Tilde => {
                    return !child_val;
                }
                TokType::Exclamation => {
                    if child_val == 0 {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                _ => panic!("Expected Unary Operator, found {:?}", op),
            }
        }
        NodeType::Const(val) => {
            return val * 1;
        }
        _ => return compute_const(tree.child.get(0).unwrap()),
    }
}
pub fn gen_prog(tree: &ParseNode) -> String {
    let p = "        ".to_string();

    // iter every function node
    let mut prog_body = String::new();
    let index_map: HashMap<String, isize> = HashMap::new();
    let mut global_variable_scope: HashSet<String> = HashSet::new();
    let idx: isize = 0;
    for it in tree.child.iter() {
        match &it.entry {
            NodeType::Declare(var_name, DataType::I64) => {
                // record it in the scope, index_map,
                global_variable_scope.insert(var_name.to_string());
                if (it.child.is_empty()) {
                    // uninitialized global variable
                    // just put them in .comm
                    // now we use value has 8 bytes by default.
                    // XXX: should be vary-length based on the data type.
                    prog_body.push_str(&format!("{}.comm {}, 8, 8\n", p, var_name, ))
                } else {
                    let val = compute_const(&it.child.get(0).unwrap());
                    prog_body.push_str(&format!(
                        "{}.globl	{}\n\
                         {}.data\n\
                         {}.align 8\n\
                         {}.type	{}, @object\n\
                         {}.size	{}, 8\n\
                         {}:\n\
                         {}.long	{}\n",
                        p, var_name, p, p, p, var_name, p, var_name, var_name, p, val
                    ));
                }
            }
            NodeType::Declare(var_name, DataType::Arr64(len)) => {
                global_variable_scope.insert(var_name.to_string());
                prog_body.push_str(&format!("{}.comm {}, {}, 32\n", p, var_name, len * 8));
            }
            NodeType::Fn(fn_name, var_list_opt) => {
                let fn_prologue = gen_fn_prologue(fn_name.to_string());
                let fn_epilogue = gen_fn_epilogue();
                // cause in function, we have to pass the offset of argument and scope contains argument
                // to function body
                let CALL_BY_FUNCTION = true;
                let mut param_offset = 16; // EBP + 16 (old EBP at 0, return address at 8)
                let mut index_map: HashMap<String, isize> = HashMap::new();
                let mut scope: HashMap<String, bool> = HashMap::new();
                match var_list_opt {
                    Some(var_list) => {
                        for var in var_list {
                            index_map.insert(var.to_string(), param_offset);
                            scope.insert(var.to_string(), true);
                            param_offset += 8;
                        }
                    }
                    None => {}
                }
                let fn_body = &gen_block(
                    it,
                    &index_map,
                    &scope,
                    idx,
                    None,
                    None,
                    true,
                    CALL_BY_FUNCTION,
                    &global_variable_scope,
                );
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
    scope: &HashMap<String, bool>, // 1 -> function argument, 0 -> local variables
    idx: isize,
    lbb: &str,
    leb: &str,
    loop_in_label: Option<&str>,
    loop_out_label: Option<&str>,
    global_variable_scope: &HashSet<String>,
) -> (HashMap<String, isize>, HashMap<String, bool>, isize, String) {
    // println!("in gen_declare with {:?}", tree.entry);
    let p = "        ";
    let mut index_map = index_map.clone();
    let mut scope = scope.clone();
    let mut idx = idx;
    match &tree.entry {
        NodeType::Declare(var_name, data_type) => {
            let get_opt = scope.get(var_name);
            match get_opt {
                Some(flag) => {
                    match flag {
                        true => {
                            // this variable is in scope, but was passed by function argument, so just shallow it
                            scope.insert(var_name.to_string(), false);
                            // println!("scope after insert: {:?}", scope);
                            index_map.insert(var_name.to_string(), idx - 8);
                            idx -= 8;
                        }
                        false => {
                            panic!(
                                "Error: redeclaration of variable `{}` in the same scope",
                                var_name
                            );
                        }
                    }
                }
                None => {
                    // not declared
                    scope.insert(var_name.to_string(), false);
                    // println!("scope after insert: {:?}", scope);
                    index_map.insert(var_name.to_string(), idx - 8);
                    idx -= 8;
                }
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
                    &global_variable_scope,
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

pub fn gen_for(
    tree: &ParseNode,
    index_map: &HashMap<String, isize>,
    idx: isize,
    global_variable_scope: &HashSet<String>,
) -> String {
    let p = "        ".to_string();
    let label_begin_loop = gen_labels("BFOR".to_string());
    let label_end_loop = gen_labels("EFOR".to_string());

    let mut index_map = index_map.clone();
    let mut idx: isize = idx;
    // now in a new block now
    let mut scope: HashMap<String, bool> = HashMap::new();
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
                &global_variable_scope,
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
                &global_variable_scope,
            );
            let post_exp = gen_stmt(
                tree.child.get(2).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                &global_variable_scope,
            );
            let stmt = gen_block(
                tree.child.get(3).unwrap(),
                &index_map,
                &scope,
                idx,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                true,
                false,
                &global_variable_scope,
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
            //let b_deallocate = 8 * scope.len();
            let mut b_deallocate = 0;
            for (_, val) in scope.iter() {
                if (*val == false) {
                    b_deallocate += 8;
                }
            }
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
                 {}addq ${}, %rsp # for out clear block\n",
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
                &global_variable_scope,
            );
            let condition = gen_stmt(
                tree.child.get(1).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                &global_variable_scope,
            );
            let post_exp = gen_stmt(
                tree.child.get(2).unwrap(),
                &index_map,
                idx,
                &label_begin_loop,
                &label_end_loop,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                &global_variable_scope,
            );
            let stmt = gen_block(
                tree.child.get(3).unwrap(),
                &index_map,
                &scope,
                idx,
                Some(&label_begin_loop),
                Some(&label_end_loop),
                true,
                false,
                &global_variable_scope,
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
            // let b_deallocate = 8 * scope.len();
            let mut b_deallocate = 0;
            for (_, val) in scope.iter() {
                if (*val == false) {
                    b_deallocate += 8;
                }
            }
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
                 {}addq ${}, %rsp # for out clear stack\n",
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
    scope: &HashMap<String, bool>,
    idx: isize,
    loop_in_label: Option<&str>,
    loop_out_label: Option<&str>,
    flag: bool,
    call_by_fn: bool,
    global_variable_scope: &HashSet<String>,
) -> String {
    let p = "        ".to_string(); // 8 white spaces
    let label_begin_block = gen_labels("BB".to_string());
    let label_end_block = gen_labels("EB".to_string());
    // iter every block
    let mut stmts = String::new();
    let mut index_map = index_map.clone();
    let mut idx: isize = idx;
    let mut current_scope: HashMap<String, bool> = scope.clone();
    if call_by_fn == false {
        current_scope = HashMap::new();
    }
    // let mut scope: HashSet<String> = HashSet::new();

    for it in &tree.child {
        // iter through every block-item
        match &it.entry {
            NodeType::Declare(_var_name, DataType::I64) => {
                let (index_map_new, scope_new, idx_new, s) = gen_declare(
                    it,
                    &index_map,
                    &current_scope,
                    idx,
                    &label_begin_block,
                    &label_end_block,
                    loop_in_label,
                    loop_out_label,
                    &global_variable_scope,
                );
                index_map = index_map_new.clone();
                idx = idx_new;
                current_scope = scope_new.clone();
                stmts.push_str(&s);
            }
            NodeType::Stmt(StmtType::Compound) => {
                stmts.push_str(&gen_block(
                    it,
                    &index_map,
                    &current_scope,
                    idx,
                    loop_in_label,
                    loop_out_label,
                    true,
                    false, // call by  function not true
                    &global_variable_scope,
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
                    &global_variable_scope,
                );
                stmts.push_str(&s);
            }
        }
    }
    let mut b_deallocate = 0;
    for (_, val) in current_scope.iter() {
        if *val == false {
            b_deallocate += 8;
        }
    }

    // println!("scope : {:?}, deallocate = {}", current_scope, b_deallocate);
    // let b_deallocate = match flag {
    //     true => 8 * scope.len(),
    //     false => 0,
    // };
    // let b_deallocate = 8 * scope.len(); // deallocate stack
    format!(
        "{}:\n\
         {}\
         {}:\n\
         {}addq ${}, %rsp # block out\n",
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
    global_variable_scope: &HashSet<String>,
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
                    &global_variable_scope,
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
                    &global_variable_scope,
                );
                let e2_as = gen_stmt(
                    tree.child.get(1).expect("conditional expression no e2"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                    &global_variable_scope,
                );
                let e3_as = gen_stmt(
                    tree.child.get(2).expect("conditional expression no e3"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                    &global_variable_scope,
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
        NodeType::FnCall(fn_name) => {
            // iter every expression in reverse direction
            // and then push them in stack
            let mut s: String = String::new();
            for it in tree.child.iter().rev() {
                // generate expression
                s.push_str(&gen_stmt(
                    it,
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                    &global_variable_scope,
                ));
                // pushq
                s.push_str(&format!("{}pushq %rax\n", p));
            }
            // call the function
            s.push_str(&format!("{}call {}\n", p, fn_name));
            // after the callee function returns, remove the arguments from stack
            s.push_str(&format!(
                "{}addq ${}, %rsp # remove the arguments\n",
                p,
                8 * tree.child.len()
            ));
            s
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
                    &global_variable_scope,
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
                    &global_variable_scope,
                );
                let s1_as = gen_stmt(
                    tree.child.get(1).expect("conditional node no s1"),
                    index_map,
                    idx,
                    lbb,
                    leb,
                    loop_in_label,
                    loop_out_label,
                    &global_variable_scope,
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
                        &global_variable_scope,
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
                &global_variable_scope,
            ),
            StmtType::Continue => match loop_in_label {
                Some(l) => format!("{}jmp {} # Continue\n", p, l),
                None => panic!("Continue should be in the loop scope"),
            },
            StmtType::Break => match loop_out_label {
                Some(l) => format!("{}jmp {} # Break\n", p, l),
                None => panic!("Break shoule be in the loop scope"),
            },
            StmtType::For | StmtType::ForDecl => {
                gen_for(tree, index_map, idx, &global_variable_scope)
            }
            StmtType::Do => {
                // LBB.
                // stmt
                // exp
                // cmpq $1, %rax
                // je  LBB
                // LEB
                let lbb = gen_labels("BDO".to_string());
                let leb = gen_labels("EDO".to_string());
                let scope: HashMap<String, bool> = HashMap::new();
                let stmts = gen_block(
                    tree.child.get(0).unwrap(),
                    index_map,
                    &scope,
                    idx,
                    loop_in_label,
                    loop_out_label,
                    true,
                    false,
                    &global_variable_scope,
                ); // should enter a new scope
                let exp = gen_stmt(
                    tree.child.get(1).unwrap(),
                    index_map,
                    idx,
                    &lbb,
                    &leb,
                    Some(&lbb),
                    Some(&leb),
                    &global_variable_scope,
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
                let scope: HashMap<String, bool> = HashMap::new();
                let exp = gen_stmt(
                    tree.child.get(0).unwrap(),
                    index_map,
                    idx,
                    &lbb,
                    &leb,
                    Some(&lbb),
                    Some(&leb),
                    &global_variable_scope,
                );
                let stmts = gen_block(
                    tree.child.get(1).unwrap(),
                    index_map,
                    &scope,
                    idx,
                    Some(&lbb),
                    Some(&leb),
                    true,
                    false,
                    &global_variable_scope,
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
                let scope: HashMap<String, bool> = HashMap::new();
                gen_block(
                    tree,
                    index_map,
                    &scope,
                    idx,
                    loop_in_label,
                    loop_out_label,
                    true,
                    false,
                    &global_variable_scope,
                )
            }
        },
        NodeType::ArrayRef(var_name) => {
            let get_index = gen_stmt(
                tree.child
                    .get(0)
                    .expect("Statement::Declare Node has no child"),
                index_map,
                idx,
                lbb,
                leb,
                loop_in_label,
                loop_out_label,
                &global_variable_scope,
            );
            // get index => rdx,
            // movq array_index var@GOTPCREL(%rip) => %rbx
            // movq (%rbx, rdx, data size), %rax
            format!(
                "{}\
                 {}movq %rax, %rdx\n\
                 {}movq {}@GOTPCREL(%rip), %rbx\n\
                 {}movq (%rbx, %rdx, 8), %rax\n",
                get_index,
                p,
                p, var_name,
                p,
            )
        },
        NodeType::AssignNode(var_name, true) => {
            match index_map.get(var_name) {
                None => {
                    // not in current scope, try to search global scope
                    match global_variable_scope.contains(var_name) {
                        true => {
                            // declared in global scope, that's ok
                            let get_index = gen_stmt(
                                tree.child
                                    .get(0)
                                    .expect("Statement::Declare Node has no child"),
                                index_map,
                                idx,
                                lbb,
                                leb,
                                loop_in_label,
                                loop_out_label,
                                &global_variable_scope,
                            );
                            let get_res = gen_stmt(
                                tree.child.get(1).unwrap(),
                                index_map,
                                idx,
                                lbb,
                                leb,
                                loop_in_label,
                                loop_out_label,
                                &global_variable_scope,
                            );
                            // movq array_index var@GOTPCREL(%rip) => %rbx
                            // get index => rdx,
                            // get res => rax
                            // movq %rax, (%rbx, rdx, data size)
                            format!(
                                "{}\
                                 {}movq %rax, %rdx\n\
                                 {}\
                                 {}movq {}@GOTPCREL(%rip), %rbx\n\
                                 {}movq %rax, (%rbx, %rdx, 8)\n",
                                get_index,
                                p,
                                get_res,
                                p, var_name,
                                p,
                            )
                        }
                        false => {
                            // Not declared before, that's not ok
                            panic!("Error: Use un-declared variable `{}`", var_name)
                        }
                    }
                }
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
                        &global_variable_scope,
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
            }
        }
        NodeType::AssignNode(var_name, false) => {
            // assign to int variable
            match index_map.get(var_name) {
                None => {
                    // not in current scope, try to search global scope
                    match global_variable_scope.contains(var_name) {
                        true => {
                            // declared in global scope, that's ok
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
                                &global_variable_scope,
                            );
                            format!(
                                "{}\
                                 {}movq %rax, {}(%rip)\n",
                                e1, p, var_name
                            )
                        }
                        false => {
                            // Not declared before, that's not ok
                            panic!("Error: Use un-declared variable `{}`", var_name)
                        }
                    }
                }
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
                        &global_variable_scope,
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
                    &global_variable_scope,
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
                    &global_variable_scope,
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
                    &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                            &global_variable_scope,
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
                            &global_variable_scope,
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
                            &global_variable_scope,
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
                            &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                        &global_variable_scope,
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
                None => {
                    // try to search global scope
                    match global_variable_scope.contains(var_name) {
                        true => {
                            // in global scope
                            let var_offset = var_name;
                            format!("{}movq {}(%rip), %rax\n", p, var_offset)
                        }
                        false => panic!(format!("Use of undeclared variable `{}`", var_name)),
                    }
                }
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
                    &global_variable_scope,
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
            &global_variable_scope,
        ),
        _ => panic!(format!(
            "Node `{:?}` not implemented in gen::gen_stmt()\n",
            &tree.entry
        )),
    }
}
