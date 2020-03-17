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
use crate::ast::ParseNode;
use crate::lexer;
use crate::symtable;

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

/// Semantics analysis driver
/// # Args:
/// * `ParseNode` : root of the parse tree
/// * `c_src_name`: input file name
///
/// # Return
/// * Ok -> Ok(())
/// * Err -> Err(msg)
pub fn sema_driver(_tree: &ParseNode, _c_src_name: &str) -> Result<(), String> {
    return Ok(());
}
