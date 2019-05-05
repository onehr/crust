//     Copyright 2019 Haoran Wang
//
//     Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.
// ------------------------------------------------------------------------
//! sema.rs : semantis checker for crust compiler tool-chain
//!
//! # rules
//!
//! * type should match when assign
//! * different type should invoke implicit cast
//! * declare before use
//! * argument type should match when calling a function
// ------------------------------------------------------------------------
use crate::ast::{NodeType, ParseNode};
use crate::lexer;
use crate::symtable;
use std::collections::HashMap;
type SymbolTable = HashMap<String, symtable::SymbolRecord>;

pub fn judge_cast(
    _to_type: &symtable::TypeExpression,
    _from_type: &symtable::TypeExpression,
) -> bool {
    // TODO: should finish a judge function:
    //       judge whether can we use type_name to cast the cast_expression
    //       most situations should raise error, like we can not write (struct) int, etc..
    //       now just return true

    return true;
}

pub fn judge_combine_type(
    _l_type: &symtable::TypeExpression,
    r_type: &symtable::TypeExpression,
    _op: &lexer::TokType,
) -> (bool, symtable::TypeExpression) {
    // TODO: now just return true and the r_type
    //       need to judge whether we can combine two types and return the new type.

    return (true, r_type.clone());
}

pub fn judge_type_same(
    _l_type: &symtable::TypeExpression,
    _r_type: &symtable::TypeExpression,
) -> bool {
    // TODO: now just return true

    return true;
}

pub fn implicit_type_cast(
    l_type: &symtable::TypeExpression,
    _r_type: &symtable::TypeExpression,
) -> Result<symtable::TypeExpression, String> {
    // TODO: implicit convert r_type to l_type, if able then return TypeExpression,
    //       else return Err. Now just simply return l_type.

    return Ok(l_type.clone());
}

fn type_extract(node: &ParseNode, idx: usize) -> Result<symtable::BaseType, String> {
    let type_exp = &node.type_exp;

    if idx >= type_exp.val.len() {
        return Err(format!("type extract error, out of val index"));
    }

    return Ok(type_exp.val.get(idx).unwrap().clone());
}

/// generate the symbol table from the abstract syntax tree.
fn sema_check(tree: &ParseNode, env: &SymbolTable) -> Result<SymbolTable, String> {
    // XXX: whether too many clones will harm the performance of compiler.
    //      need to benchmark
    if tree.entry == NodeType::Declaration {
        // it should have two children, first is type, second is identifier name
        let mut env = env.clone();
        // get the type
        let type_: symtable::TypeExpression;
        match tree.child.get(0) {
            // shouldn't occur
            None => {
                return Err(format!("No type specifier"));
            }
            Some(t) => {
                // type_ = type_extract(t, 0)?;
                type_ = t.type_exp.clone();
            }
        }
        // get the identifier
        let identifier: String;
        match tree.child.get(1) {
            None => {
                return Err(format!("No identifier"));
            }
            Some(t) => match type_extract(t, 0) {
                Ok(symtable::BaseType::Identifier(name)) => {
                    identifier = name.clone();
                }
                Err(msg) => {
                    return Err(msg);
                }
                _ => {
                    return Err(format!("Not able to extract "));
                }
            },
        }

        println!("try to search the map to find whether it's declared before");
        println!("env: {:?}", env);
        match env.get(&identifier) {
            Some(_) => {
                println!("got identifier {}", identifier);
                return Err(format!("Error: re-declare identifier {}", identifier));
            }
            None => {}
        }
        let mut attr = symtable::SymbolAttr::new();
        attr.set_base_type(type_);
        // println!("id: {:?}, attr: {:?}", identifier, attr);
        let record = symtable::SymbolRecord::new(identifier.clone(), attr);
        // insert the name - record to env
        env.insert(identifier, record);
        // println!("env: {:?}", env);
        return Ok(env.clone());
    } else {
        let mut now_env = env.clone();
        for it in tree.child.iter() {
            now_env = sema_check(&it, &now_env)?;
        }
        return Ok(now_env.clone());
    }
}

/// Semantics analysis driver
/// # Args:
/// * `ParseNode` : root of the parse tree
/// * `c_src_name`: input file name
///
/// # Return
/// * Ok -> Ok(())
/// * Err -> Err(msg)
pub fn sema_driver(tree: &ParseNode, _c_src_name: &str) -> Result<(), String> {
    let env = SymbolTable::new();
    sema_check(tree, &env)?;

    return Ok(());
}
