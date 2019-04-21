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
// symtable.rs: symbol table for identifiers.
// ------------------------------------------------------------------------

use crate::opts::StopStage;

const X86_64_CHAR_BYTES: u64 = 1;
const X86_64_SHORT_BYTES: u64 = 2;
const X86_64_INT_BYTES: u64 = 4;
const X86_64_LONG_BYTES: u64 = 8;

const NUM_REG: usize = 16;

const B64_REG_NAMES: [&str; NUM_REG] = [
    "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "r8", "r9", "r10", "r11", "r12", "r13",
    "r14", "r15",
];
const B32_REG_NAMES: [&str; NUM_REG] = [
    "eax", "ebx", "ecx", "edx", "esi", "edi", "ebp", "esp", "r8d", "r9d", "r10d", "r11d", "r12d",
    "r13d", "r14d", "r15d",
];
const B16_REG_NAMES: [&str; NUM_REG] = [
    "ax", "bx", "cx", "dx", "si", "di", "bp", "sp", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w",
    "r14w", "r15w",
];
const B8_REG_NAMES: [&str; NUM_REG] = [
    "al", "bl", "cl", "dl", "sil", "dil", "bpl", "spl", "r8b", "r9b", "r10b", "r11b", "r12b",
    "r13b", "r14b", "r15b",
];

#[derive(PartialEq, Clone, Debug)]
pub struct SymbolRecord {
    pub name: String,
    pub attr: SymbolAttr,
}

impl SymbolRecord {
    pub fn new(s: String, a: SymbolAttr) -> SymbolRecord {
        SymbolRecord {
            name: s.clone(),
            attr: a,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum BaseType {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    Bool,
    Complex,
    Imaginary,
    Pointer,
    Function,
    Array(u64), // len
    Struct,
    Union,
}

/// struct: TypeExpressionTree
///
/// # Note:
///
/// In semantics analysis, the semantics checker should build a TypeExpressionTree.
/// to make type checking
#[derive(PartialEq, Clone, Debug)]
pub struct TypeExpression{
    pub val: Option<BaseType>,
    pub child: Vec<TypeExpression>,
}

impl TypeExpression{
    pub fn new() -> TypeExpression{
        TypeExpression{
            val: None,
            child: Vec::new(),
        }
    }
    pub fn new_val(s: BaseType) -> TypeExpression{
        TypeExpression{
            val: Some(s.clone()),
            child: Vec::new(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
enum StorageClass {
    Local,
    Static,
    Global,
}

#[derive(PartialEq, Clone, Debug)]
pub struct SymbolAttr {
    volatile: bool,              // Asynchronously accessed.
    size: u64,                   // size in bytes.
    boundary: u64,               // alignment in bytes.
    base_type: TypeExpression,         // base type in source language.
    n_elements: u64,             // number of elements.
    register: bool,              // whether the value is in register.
    reg: u64,                    // index of the name of register which contains the value.
    base_reg: u64, // index of the name of register used to calculate the symbol's address.
    storage_class: StorageClass, // `local`, `static`, `global`
    fn_parameter: bool,   // true: a function parameter
    // loc: SourceLoc// TODO: add source code location
}

impl SymbolAttr {
    pub fn new() -> SymbolAttr {
        SymbolAttr {
            volatile: false,
            size: X86_64_INT_BYTES,
            boundary: X86_64_INT_BYTES,
            base_type: TypeExpression::new_val(BaseType::Int),
            n_elements: 1,
            register: false,
            reg: 0,
            base_reg: 0,
            storage_class: StorageClass::Local,
            fn_parameter: false,
        }
    }
    pub fn set_volatile(&mut self, val: bool) {
        self.volatile = val;
    }
    pub fn set_size(&mut self, val: u64) {
        self.size = val;
    }
    pub fn set_boundary(&mut self, val: u64) {
        self.boundary = val;
    }
    pub fn set_base_type(&mut self, val: TypeExpression) {
        self.base_type = val.clone();
    }
    pub fn set_n_elements(&mut self, val: u64) {
        self.n_elements = val;
    }
    pub fn set_register(&mut self, val: bool) {
        self.register = val;
    }
    pub fn set_reg(&mut self, idx: u64) {
        self.reg = idx;
    }
    pub fn set_base_reg(&mut self, idx: u64) {
        self.base_reg = idx;
    }
    pub fn set_storage_class(&mut self, class: StorageClass) {
        self.storage_class = class;
    }
    pub fn set_fn_parameter(&mut self, val: bool) {
        self.fn_parameter = val;
    }

    pub fn get_volatile(&self) -> bool {
        self.volatile
    }
    pub fn get_size(&self) -> u64 {
        self.size
    }
    pub fn get_boundary(&self) -> u64 {
        self.boundary
    }
    pub fn get_base_type(&self) -> TypeExpression {
        self.base_type.clone()
    }
    pub fn get_n_elements(&self) -> u64 {
        self.n_elements
    }
    pub fn get_register(&self) -> bool {
        self.register
    }
    pub fn get_reg(&self) -> u64 {
        self.reg
    }
    pub fn get_basereg(&self) -> u64 {
        self.base_reg
    }
    pub fn get_storage_class(&self) -> StorageClass {
        self.storage_class.clone()
    }
    pub fn get_fn_parameter(&self) -> bool {
        self.fn_parameter
    }
}
