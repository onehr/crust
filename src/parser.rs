use crate::lexer;

// TODO:
// Trying to implement a better parser to support c11 full standard
// assmue the the lexer can contains all keywords in the c11 standard.
// The lexer should produce the token into 5 types:
//    Keywords (reserved keywords)
//    Identifiers
//    Constants
//    Punctuators
//    Operators

// Token type should contains
//    IDENTIFIER
//    I_CONSTANT F_CONSTANT STRING_LITERAL
//    FUNC_NAME
//    SIZEOF
//    PTR_OP INC_OP DEC_OP LEFT_OP RIGHT_OP LE_OP GE_OP EQ_OP NE_OP
//    AND_OP OR_OP MUL_ASSIGN DIV_ASSIGN MOD_ASSIGN ADD_ASSIGN
//    SUB_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN AND_ASSIGN
//    XOR_ASSIGN OR_ASSIGN
//    TYPEDEF_NAME ENUMERATION_CONSTANT
//    TYPEDEF EXTERN STATIC AUTO REGISTER INLINE
//    CONST RESTRICT VOLATILE
//    BOOL CHAR SHORT INT LONG SIGNED UNSIGNED FLOAT DOUBLE VOID
//    COMPLEX IMAGINARY
//    STRUCT UNION ENUM ELLIPSIS
//    CASE DEFAULT IF ELSE SWITCH WHILE DO FOR GOTO CONTINUE BREAK RETURN
//    ALIGNAS ALIGNOF ATOMIC GENERIC NORETURN STATIC_ASSERT THREAD_LOCAL

// primary_expression
// 	: IDENTIFIER
// 	| constant
// 	| string
// 	| '(' expression ')'
// 	| generic_selection
// 	;

// constant
// 	: I_CONSTANT		/* includes character_constant */
// 	| F_CONSTANT
// 	| ENUMERATION_CONSTANT	/* after it has been defined as such */
// 	;

// enumeration_constant		/* before it has been defined as such */
// 	: IDENTIFIER
// 	;

// string
// 	: STRING_LITERAL
// 	| FUNC_NAME
// 	;

// generic_selection
// 	: GENERIC '(' assignment_expression ',' generic_assoc_list ')'
// 	;

// generic_assoc_list
// 	: generic_association
// 	| generic_assoc_list ',' generic_association
// 	;

// generic_association
// 	: type_name ':' assignment_expression
// 	| DEFAULT ':' assignment_expression
// 	;

// postfix_expression
// 	: primary_expression
// 	| postfix_expression '[' expression ']'
// 	| postfix_expression '(' ')'
// 	| postfix_expression '(' argument_expression_list ')'
// 	| postfix_expression '.' IDENTIFIER
// 	| postfix_expression PTR_OP IDENTIFIER
// 	| postfix_expression INC_OP
// 	| postfix_expression DEC_OP
// 	| '(' type_name ')' '{' initializer_list '}'
// 	| '(' type_name ')' '{' initializer_list ',' '}'
// 	;

// argument_expression_list
// 	: assignment_expression
// 	| argument_expression_list ',' assignment_expression
// 	;

// unary_expression
// 	: postfix_expression
// 	| INC_OP unary_expression
// 	| DEC_OP unary_expression
// 	| unary_operator cast_expression
// 	| SIZEOF unary_expression
// 	| SIZEOF '(' type_name ')'
// 	| ALIGNOF '(' type_name ')'
// 	;

// unary_operator
// 	: '&'
// 	| '*'
// 	| '+'
// 	| '-'
// 	| '~'
// 	| '!'
// 	;

// cast_expression
// 	: unary_expression
// 	| '(' type_name ')' cast_expression
// 	;

// multiplicative_expression
// 	: cast_expression
// 	| multiplicative_expression '*' cast_expression
// 	| multiplicative_expression '/' cast_expression
// 	| multiplicative_expression '%' cast_expression
// 	;

// additive_expression
// 	: multiplicative_expression
// 	| additive_expression '+' multiplicative_expression
// 	| additive_expression '-' multiplicative_expression
// 	;

// shift_expression
// 	: additive_expression
// 	| shift_expression LEFT_OP additive_expression
// 	| shift_expression RIGHT_OP additive_expression
// 	;

// relational_expression
// 	: shift_expression
// 	| relational_expression '<' shift_expression
// 	| relational_expression '>' shift_expression
// 	| relational_expression LE_OP shift_expression
// 	| relational_expression GE_OP shift_expression
// 	;

// equality_expression
// 	: relational_expression
// 	| equality_expression EQ_OP relational_expression
// 	| equality_expression NE_OP relational_expression
// 	;

// and_expression
// 	: equality_expression
// 	| and_expression '&' equality_expression
// 	;

// exclusive_or_expression
// 	: and_expression
// 	| exclusive_or_expression '^' and_expression
// 	;

// inclusive_or_expression
// 	: exclusive_or_expression
// 	| inclusive_or_expression '|' exclusive_or_expression
// 	;

// logical_and_expression
// 	: inclusive_or_expression
// 	| logical_and_expression AND_OP inclusive_or_expression
// 	;

// logical_or_expression
// 	: logical_and_expression
// 	| logical_or_expression OR_OP logical_and_expression
// 	;

// conditional_expression
// 	: logical_or_expression
// 	| logical_or_expression '?' expression ':' conditional_expression
// 	;

// assignment_expression
// 	: conditional_expression
// 	| unary_expression assignment_operator assignment_expression
// 	;

// assignment_operator
// 	: '='
// 	| MUL_ASSIGN
// 	| DIV_ASSIGN
// 	| MOD_ASSIGN
// 	| ADD_ASSIGN
// 	| SUB_ASSIGN
// 	| LEFT_ASSIGN
// 	| RIGHT_ASSIGN
// 	| AND_ASSIGN
// 	| XOR_ASSIGN
// 	| OR_ASSIGN
// 	;

// expression
// 	: assignment_expression
// 	| expression ',' assignment_expression
// 	;

// constant_expression
// 	: conditional_expression	/* with constraints */
// 	;

// declaration
// 	: declaration_specifiers ';'
// 	| declaration_specifiers init_declarator_list ';'
// 	| static_assert_declaration
// 	;

// declaration_specifiers
// 	: storage_class_specifier declaration_specifiers
// 	| storage_class_specifier
// 	| type_specifier declaration_specifiers
// 	| type_specifier
// 	| type_qualifier declaration_specifiers
// 	| type_qualifier
// 	| function_specifier declaration_specifiers
// 	| function_specifier
// 	| alignment_specifier declaration_specifiers
// 	| alignment_specifier
// 	;

// init_declarator_list
// 	: init_declarator
// 	| init_declarator_list ',' init_declarator
// 	;

// init_declarator
// 	: declarator '=' initializer
// 	| declarator
// 	;

// storage_class_specifier
// 	: TYPEDEF	/* identifiers must be flagged as TYPEDEF_NAME */
// 	| EXTERN
// 	| STATIC
// 	| THREAD_LOCAL
// 	| AUTO
// 	| REGISTER
// 	;

// type_specifier
// 	: VOID
// 	| CHAR
// 	| SHORT
// 	| INT
// 	| LONG
// 	| FLOAT
// 	| DOUBLE
// 	| SIGNED
// 	| UNSIGNED
// 	| BOOL
// 	| COMPLEX
// 	| IMAGINARY	  	/* non-mandated extension */
// 	| atomic_type_specifier
// 	| struct_or_union_specifier
// 	| enum_specifier
// 	| TYPEDEF_NAME		/* after it has been defined as such */
// 	;

// struct_or_union_specifier
// 	: struct_or_union '{' struct_declaration_list '}'
// 	| struct_or_union IDENTIFIER '{' struct_declaration_list '}'
// 	| struct_or_union IDENTIFIER
// 	;

// struct_or_union
// 	: STRUCT
// 	| UNION
// 	;

// struct_declaration_list
// 	: struct_declaration
// 	| struct_declaration_list struct_declaration
// 	;

// struct_declaration
// 	: specifier_qualifier_list ';'	/* for anonymous struct/union */
// 	| specifier_qualifier_list struct_declarator_list ';'
// 	| static_assert_declaration
// 	;

// specifier_qualifier_list
// 	: type_specifier specifier_qualifier_list
// 	| type_specifier
// 	| type_qualifier specifier_qualifier_list
// 	| type_qualifier
// 	;

// struct_declarator_list
// 	: struct_declarator
// 	| struct_declarator_list ',' struct_declarator
// 	;

// struct_declarator
// 	: ':' constant_expression
// 	| declarator ':' constant_expression
// 	| declarator
// 	;

// enum_specifier
// 	: ENUM '{' enumerator_list '}'
// 	| ENUM '{' enumerator_list ',' '}'
// 	| ENUM IDENTIFIER '{' enumerator_list '}'
// 	| ENUM IDENTIFIER '{' enumerator_list ',' '}'
// 	| ENUM IDENTIFIER
// 	;

// enumerator_list
// 	: enumerator
// 	| enumerator_list ',' enumerator
// 	;

// enumerator	/* identifiers must be flagged as ENUMERATION_CONSTANT */
// 	: enumeration_constant '=' constant_expression
// 	| enumeration_constant
// 	;

// atomic_type_specifier
// 	: ATOMIC '(' type_name ')'
// 	;

// type_qualifier
// 	: CONST
// 	| RESTRICT
// 	| VOLATILE
// 	| ATOMIC
// 	;

// function_specifier
// 	: INLINE
// 	| NORETURN
// 	;

// alignment_specifier
// 	: ALIGNAS '(' type_name ')'
// 	| ALIGNAS '(' constant_expression ')'
// 	;

// declarator
// 	: pointer direct_declarator
// 	| direct_declarator
// 	;

// direct_declarator
// 	: IDENTIFIER
// 	| '(' declarator ')'
// 	| direct_declarator '[' ']'
// 	| direct_declarator '[' '*' ']'
// 	| direct_declarator '[' STATIC type_qualifier_list assignment_expression ']'
// 	| direct_declarator '[' STATIC assignment_expression ']'
// 	| direct_declarator '[' type_qualifier_list '*' ']'
// 	| direct_declarator '[' type_qualifier_list STATIC assignment_expression ']'
// 	| direct_declarator '[' type_qualifier_list assignment_expression ']'
// 	| direct_declarator '[' type_qualifier_list ']'
// 	| direct_declarator '[' assignment_expression ']'
// 	| direct_declarator '(' parameter_type_list ')'
// 	| direct_declarator '(' ')'
// 	| direct_declarator '(' identifier_list ')'
// 	;

// pointer
// 	: '*' type_qualifier_list pointer
// 	| '*' type_qualifier_list
// 	| '*' pointer
// 	| '*'
// 	;

// type_qualifier_list
// 	: type_qualifier
// 	| type_qualifier_list type_qualifier
// 	;

// parameter_type_list
// 	: parameter_list ',' ELLIPSIS
// 	| parameter_list
// 	;

// parameter_list
// 	: parameter_declaration
// 	| parameter_list ',' parameter_declaration
// 	;

// parameter_declaration
// 	: declaration_specifiers declarator
// 	| declaration_specifiers abstract_declarator
// 	| declaration_specifiers
// 	;

// identifier_list
// 	: IDENTIFIER
// 	| identifier_list ',' IDENTIFIER
// 	;

// type_name
// 	: specifier_qualifier_list abstract_declarator
// 	| specifier_qualifier_list
// 	;

// abstract_declarator
// 	: pointer direct_abstract_declarator
// 	| pointer
// 	| direct_abstract_declarator
// 	;

// direct_abstract_declarator
// 	: '(' abstract_declarator ')'
// 	| '[' ']'
// 	| '[' '*' ']'
// 	| '[' STATIC type_qualifier_list assignment_expression ']'
// 	| '[' STATIC assignment_expression ']'
// 	| '[' type_qualifier_list STATIC assignment_expression ']'
// 	| '[' type_qualifier_list assignment_expression ']'
// 	| '[' type_qualifier_list ']'
// 	| '[' assignment_expression ']'
// 	| direct_abstract_declarator '[' ']'
// 	| direct_abstract_declarator '[' '*' ']'
// 	| direct_abstract_declarator '[' STATIC type_qualifier_list assignment_expression ']'
// 	| direct_abstract_declarator '[' STATIC assignment_expression ']'
// 	| direct_abstract_declarator '[' type_qualifier_list assignment_expression ']'
// 	| direct_abstract_declarator '[' type_qualifier_list STATIC assignment_expression ']'
// 	| direct_abstract_declarator '[' type_qualifier_list ']'
// 	| direct_abstract_declarator '[' assignment_expression ']'
// 	| '(' ')'
// 	| '(' parameter_type_list ')'
// 	| direct_abstract_declarator '(' ')'
// 	| direct_abstract_declarator '(' parameter_type_list ')'
// 	;

// initializer
// 	: '{' initializer_list '}'
// 	| '{' initializer_list ',' '}'
// 	| assignment_expression
// 	;

// initializer_list
// 	: designation initializer
// 	| initializer
// 	| initializer_list ',' designation initializer
// 	| initializer_list ',' initializer
// 	;

// designation
// 	: designator_list '='
// 	;

// designator_list
// 	: designator
// 	| designator_list designator
// 	;

// designator
// 	: '[' constant_expression ']'
// 	| '.' IDENTIFIER
// 	;

// static_assert_declaration
// 	: STATIC_ASSERT '(' constant_expression ',' STRING_LITERAL ')' ';'
// 	;

// statement
// 	: labeled_statement
// 	| compound_statement
// 	| expression_statement
// 	| selection_statement
// 	| iteration_statement
// 	| jump_statement
// 	;

// labeled_statement
// 	: IDENTIFIER ':' statement
// 	| CASE constant_expression ':' statement
// 	| DEFAULT ':' statement
// 	;

// compound_statement
// 	: '{' '}'
// 	| '{'  block_item_list '}'
// 	;

// block_item_list
// 	: block_item
// 	| block_item_list block_item
// 	;

// block_item
// 	: declaration
// 	| statement
// 	;

// expression_statement
// 	: ';'
// 	| expression ';'
// 	;

// selection_statement
// 	: IF '(' expression ')' statement ELSE statement
// 	| IF '(' expression ')' statement
// 	| SWITCH '(' expression ')' statement
// 	;

// iteration_statement
// 	: WHILE '(' expression ')' statement
// 	| DO statement WHILE '(' expression ')' ';'
// 	| FOR '(' expression_statement expression_statement ')' statement
// 	| FOR '(' expression_statement expression_statement expression ')' statement
// 	| FOR '(' declaration expression_statement ')' statement
// 	| FOR '(' declaration expression_statement expression ')' statement
// 	;

// jump_statement
// 	: GOTO IDENTIFIER ';'
// 	| CONTINUE ';'
// 	| BREAK ';'
// 	| RETURN ';'
// 	| RETURN expression ';'
// 	;

// translation_unit
// 	: external_declaration
// 	| translation_unit external_declaration
// 	;

// external_declaration
// 	: function_definition
// 	| declaration
// 	;

// function_definition
// 	: declaration_specifiers declarator declaration_list compound_statement
// 	| declaration_specifiers declarator compound_statement
// 	;

// declaration_list
// 	: declaration
// 	| declaration_list declaration
// 	;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NodeType {
    Prog(String),
    // TODO: now only support int parameters
    // <function> ::= "int" <id> "(" [ "int" <id> { "," "int" <id> } ] ")" "{" {<block-item>} "}"
    Fn(String, Option<Vec<String>>),
    Stmt(StmtType),
    // <statement> ::= "return" <exp> ";"
    //               | <exp-option> ";"
    //               | "if" "(" <exp> ")" <statement> [ "else" <statement> ]
    //               | "{" { <block-item> } "}
    //               | "for" "(" <exp-option> ";" <exp-option> ";" <exp-option> ")" <statement>
    //               | "for" "(" <declaration> <exp-option> ";" <exp-option> ")" <statement>
    //               | "while" "(" <exp> ")" <statement>
    //               | "do" <statement> "while" <exp> ";"
    //               | "break" ";"
    //               | "continue" ";"
    Block, // <block> ::= <statement> | <declaration>
    Const(i64),
    StringLiteral(String, String), // data, tag
    Var(String),
    ArrayRef(String),          // referencing to array
    AssignNode(String, bool), // String -> variable name, bool -> true if this is a assign to array element
    UnExp(lexer::TokType),    // Unary Expression
    BinExp(lexer::TokType),   // Binary Operator
    Exp,                      // <exp> ::= <id> ["[" <exp> "]"] "=" <exp> | <conditional-exp>
    ExpOption,                // <exp-option> :: <exp> | ""
    ConditionalExp, // <conditional-exp> ::= <logical-or-exp> [ "?" <exp> ":" <conditional-exp> ]
    LogicalOrExp,   // <logical-or-exp> ::= <logical-and-exp> { "||" <logical-and-exp> }
    LogicalAndExp,  // <logical-and-exp> ::= <equality-exp> { "&&" <equality-exp> }
    EqualityExp,    // <EqualityExp> ::= <relational-exp> { ("!="|"==") <relational-exp> }
    RelationalExp, // <relational-exp> ::= <additive-exp> { ("<" | ">" | "<=" | ">=") <additive-exp> }
    AdditiveExp,   // <additive-exp> ::= <term> { ("+" | "-") <term> }
    Term,          // <term> ::= <factor> { ("*" | "/") <factor> }
    Factor, // <factor> ::= <function-call> | "(" <exp> ")" | <unary_op> <factor> | <int> | string | <id> "[" <exp> "]" | <id>
    FnCall(String), // <function-call> ::= id "(" [ <exp> { "," <exp> } ] ")"
    Declare(String, DataType), // <declaration> ::= "int" <id> "[" <int> "]" ";" | "int" <id> [ = <exp> ] ";"
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum DataType {
    I64,        // now int in c was translated in 64 bits int
    Arr64(i64), // int array[len]
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum StmtType {
    Return,
    Exp,
    Conditional(String),
    Compound,
    For,     // kids: exp-opion, exp-option, exp-option
    ForDecl, // kids: declaration, exp, exp-option, statement
    While,   // kids: exp, stmt
    Do,      // kids: stmt, exp
    Break,
    Continue,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParseNode {
    pub child: Vec<ParseNode>,
    pub entry: NodeType,
}

impl ParseNode {
    pub fn new() -> ParseNode {
        ParseNode {
            child: Vec::new(),
            entry: NodeType::Prog("root".to_string()),
        }
    }
}

fn p_logical_or_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut log_or_exp_node = ParseNode::new();
    log_or_exp_node.entry = NodeType::LogicalOrExp;
    // Parse <logical-and-exp> first

    // <LogicalOrExp> -> <LogicalAndExp>
    let mut pos = pos;
    let (log_and_exp_node, tmp_pos) = r#try!(p_logical_and_exp(toks, pos));
    pos = tmp_pos;
    // peek next node
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::Or {
        // only one child_node
        log_or_exp_node.child.push(log_and_exp_node);
        pos = pos - 1;
        return Ok((log_or_exp_node, pos));
    }

    // log_or_exp -> BinExp -> (left: logAndExp, right logAndExp)
    let mut lhs = log_and_exp_node;
    while *tok == lexer::TokType::Or {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(lexer::TokType::Or);

        let (rhs, tmp_pos) = r#try!(p_logical_and_exp(toks, pos));

        binexp_node.child.push(lhs);
        binexp_node.child.push(rhs);
        lhs = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    log_or_exp_node.child.push(lhs);
    pos = pos - 1;
    return Ok((log_or_exp_node, pos));
}

fn p_conditional_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // <conditional-exp> ::= <logical-or-exp> [ "?" <exp> ":" <conditional-exp> ]
    let mut conditional_exp_node = ParseNode::new();
    conditional_exp_node.entry = NodeType::ConditionalExp;
    // parse <logical-or-exp> first
    let (logical_or_exp_node, pos) = r#try!(p_logical_or_exp(toks, pos));
    conditional_exp_node.child.push(logical_or_exp_node);

    // it's optional if you got a "?"
    if toks[pos] == lexer::TokType::QuestionMark {
        // parse <exp>
        let pos = pos + 1;
        let (exp_node, pos) = r#try!(p_exp(toks, pos));

        if toks[pos] != lexer::TokType::Colon {
            return Err(format!(
                "Expected `:` in conditional expression, but got {:?} at {}",
                toks[pos], pos
            ));
        }
        let pos = pos + 1;
        // parse next <conditonal-exp>
        let (next_conditional_exp_node, pos) = r#try!(p_conditional_exp(toks, pos));
        conditional_exp_node.child.push(exp_node);
        conditional_exp_node.child.push(next_conditional_exp_node);
        return Ok((conditional_exp_node, pos));
    } else {
        return Ok((conditional_exp_node, pos));
    }
}

fn p_exp_opt(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // <exp-option> ::= <exp> | ""
    let mut exp_opt_node = ParseNode::new();
    exp_opt_node.entry = NodeType::ExpOption;
    let res = p_exp(toks, pos);
    match res {
        Ok((exp_node, pos)) => {
            // <exp>
            exp_opt_node.child.push(exp_node);
            return Ok((exp_opt_node, pos));
        }
        Err(_) => {
            // ""
            // no child, means null statement
            return Ok((exp_opt_node, pos));
        }
    }
}

fn p_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in fn: p_exp, with pos:{}", pos);
    // <exp> ::= <id> [ "[" <exp> "]" ] "=" <exp> | <conditional-exp>
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::Exp;

    let tok = &toks[pos];
    match tok {
        lexer::TokType::Identifier(var_name) => {
            // check next token is Assign
            let mut pos = pos + 1;
            let tok = &toks[pos];
            match tok {
                lexer::TokType::Assign => {
                    pos = pos + 1;
                    // something like a = 1
                    let mut assign_node = ParseNode::new();
                    assign_node.entry = NodeType::AssignNode(var_name.to_string(), false); // assign a int variable
                    let (next_exp_node, pos) = r#try!(p_exp(toks, pos));
                    assign_node.child.push(next_exp_node);
                    return Ok((assign_node, pos));
                }
                lexer::TokType::LBracket => {
                    // something like a[<exp>] = 1;
                    let back_pos = pos - 1;
                    pos = pos + 1;
                    // parse exp.
                    let (index_node, new_pos) = r#try!(p_exp(toks, pos));
                    pos = new_pos;
                    // parse ']'
                    if toks[pos] != lexer::TokType::RBracket {
                        return Err(format!(
                            "Expected ']' for bracket closing, found {:?} at {}",
                            toks[pos], pos
                        ));
                    }

                    // try '='
                    pos = pos + 1;
                    if toks[pos] != lexer::TokType::Assign {
                        pos = back_pos;
                        let (conditional_node, pos) = r#try!(p_conditional_exp(toks, pos));
                        exp_node.child.push(conditional_node);
                        return Ok((exp_node, pos));
                    }
                    pos = pos + 1;
                    // try parse exp
                    let mut assign_node = ParseNode::new();
                    assign_node.entry = NodeType::AssignNode(var_name.to_string(), true); // assign to a array element
                    let (res_node, new_pos) = r#try!(p_exp(toks, pos));
                    pos = new_pos;
                    assign_node.child.push(index_node);
                    assign_node.child.push(res_node);
                    return Ok((assign_node, pos));
                }
                _ => {
                    pos = pos - 1;
                    let (conditional_node, pos) = r#try!(p_conditional_exp(toks, pos));
                    exp_node.child.push(conditional_node);
                    return Ok((exp_node, pos));
                }
            }
        }
        _ => {
            // try <conditional-exp>
            let (cond_node, pos) = r#try!(p_conditional_exp(toks, pos));
            exp_node.child.push(cond_node);
            return Ok((exp_node, pos));
        }
    }
}

fn p_fn(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_fn with pos: {}", pos);
    // <function> ::= "int" <id> "(" ")" "{" { <statement> } "}"
    if pos >= toks.len() {
        return Err("Out of program length".to_string());
    }
    let tok = &toks[pos];
    if *tok != lexer::TokType::Kwd(lexer::KwdType::Int) {
        return Err(format!("Expected `int`, found {:?} at {}", toks[pos], pos));
    }
    let mut pos = pos + 1;

    let tok = &toks[pos];
    let mut fn_name = String::new();
    match tok {
        lexer::TokType::Identifier(n) => {
            fn_name = n.to_string();
        }
        _ => {
            return Err(format!("Expected function name, but not function name"));
        }
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LParen {
        return Err(format!("Expected `(`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;
    // XXX: add void support, now only support int arg list
    let mut arg_list: Vec<String> = Vec::new();
    let mut arg_count = 0;
    while pos < toks.len() && toks[pos] != lexer::TokType::RParen {
        // try to parse argument list
        // match int
        match &toks[pos] {
            lexer::TokType::Kwd(lexer::KwdType::Int) => {
                pos = pos + 1;
            }
            lexer::TokType::Kwd(lexer::KwdType::Void) => {
                if arg_count > 0 {
                    return Err(format!(
                        "Error: void after other argument in one function definition"
                    ));
                }
                pos = pos + 1;
                break;
            }
            _ => {
                return Err(format!("Expected `int`, found {:?} at {}", toks[pos], pos));
            }
        }
        // match identifier
        match &toks[pos] {
            lexer::TokType::Identifier(var_name) => {
                arg_list.push(var_name.to_string());
                pos = pos + 1;
            }
            _ => {
                return Err(format!(
                    "Expected identifier name, found {:?} at {}",
                    toks[pos], pos
                ));
            }
        }
        arg_count = arg_count + 1;
        // match ,
        match &toks[pos] {
            lexer::TokType::Comma => {
                pos = pos + 1;
            }
            lexer::TokType::RParen => {
                continue;
            }
            _ => {
                return Err(format!(
                    "Expected `,` or `)` at the end of one var_name, found {:?} at {}",
                    toks[pos], pos
                ));
            }
        }
        if toks[pos] == lexer::TokType::RParen {
            break;
        }
    }
    let tok = &toks[pos];
    if *tok != lexer::TokType::RParen {
        return Err(format!("Expected `)`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let tok = &toks[pos];
    if *tok != lexer::TokType::LBrace {
        return Err(format!("Expected `{{`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    let mut fn_node = ParseNode::new();
    if arg_list.is_empty() {
        fn_node.entry = NodeType::Fn(fn_name, None);
    } else {
        fn_node.entry = NodeType::Fn(fn_name, Some(arg_list));
    }

    while pos < toks.len() && toks[pos] != lexer::TokType::RBrace {
        let (block_node, tmp_pos) = r#try!(p_block(toks, pos));
        pos = tmp_pos;
        fn_node.child.push(block_node);
    }

    if pos >= toks.len() {
        return Err(format!("Missing `}}`"));
    }
    if toks[pos] != lexer::TokType::RBrace {
        return Err(format!("Expected `}}`, found {:?} at {}", toks[pos], pos));
    }
    pos = pos + 1;

    //println!("out p_fn with pos: {}", pos);
    Ok((fn_node, pos))
}

fn p_declare(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_declare with pos = {}", pos);
    let tok = &toks[pos];
    match tok {
        lexer::TokType::Kwd(lexer::KwdType::Int) => {
            // "int" <id> [ = <exp> ] ";"
            // or "int" <id> "[" <int> "]" ";"
            let pos = pos + 1;

            let tok = &toks[pos];
            match tok {
                lexer::TokType::Identifier(var_name) => {
                    let mut stmt_node = ParseNode::new();
                    stmt_node.entry = NodeType::Declare(var_name.to_string(), DataType::I64);
                    let pos = pos + 1;
                    let tok = &toks[pos];
                    match tok {
                        lexer::TokType::Assign => {
                            // parse exp
                            // e.g. int a = exp;
                            let pos = pos + 1;
                            let (exp_node, pos) = r#try!(p_exp(toks, pos));

                            let tok = &toks[pos];
                            if *tok != lexer::TokType::Semicolon {
                                return Err(format!(
                                    "Expected `;`, found {:?} at {}",
                                    toks[pos], pos
                                ));
                            }
                            let pos = pos + 1;
                            stmt_node.child.push(exp_node);
                            return Ok((stmt_node, pos));
                        }
                        lexer::TokType::Semicolon => {
                            // if just declare, but no assignment, just record the var_name
                            // e.g. int var;
                            let pos = pos + 1;
                            return Ok((stmt_node, pos));
                        }
                        lexer::TokType::LBracket => {
                            // array declare
                            // e.g. int a[100];
                            // XXX: now only just support literal array length
                            if cfg!(feature = "debug") {
                                println!("here in p_declare -> LBraket");
                            }
                            let mut declare_node = ParseNode::new();
                            let pos = pos + 1;
                            let tok = &toks[pos];
                            match tok {
                                lexer::TokType::Literal(n) => {
                                    declare_node.entry = NodeType::Declare(
                                        var_name.to_string(),
                                        DataType::Arr64(*n),
                                    );
                                    let pos = pos + 1;
                                    let tok = &toks[pos];
                                    if *tok != lexer::TokType::RBracket {
                                        return Err(format!(
                                            "Expected `]` for array declaration, found {:?} at {}",
                                            toks[pos], pos
                                        ));
                                    }

                                    let pos = pos + 1;
                                    let tok = &toks[pos];
                                    if *tok != lexer::TokType::Semicolon {
                                        return Err(format!("Expected `;` at end of array declaration, found {:?} at {}", toks[pos], pos));
                                    }
                                    let pos = pos + 1;
                                    if cfg!(feature = "debug") {
                                        println!("got declare_node: {:?}", declare_node);
                                    }
                                    return Ok((declare_node, pos));
                                }
                                _ => {
                                    return Err(format!(
                                        "Expected Array length `literal`, found {:?} at {}",
                                        toks[pos], pos
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(format!(
                                "Expected Assignment `;` or `=`, found {:?} at {}",
                                toks[pos], pos
                            ));
                        }
                    }
                }
                _ => {
                    return Err(format!(
                        "Expected identifier name, found {:?} at {}",
                        toks[pos], pos
                    ));
                }
            }
        }
        _ => {
            return Err(format!(
                "Error: Expected type definition `int`, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
}

fn p_block(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    let tok = &toks[pos];
    match tok {
        lexer::TokType::Kwd(lexer::KwdType::Int) => {
            // try to parse declare
            // let mut block_node = ParseNode::new();
            // block_node.entry = NodeType::Block;

            let (declare_node, pos) = r#try!(p_declare(toks, pos));
            // block_node.child.push(declare_node);
            // return Ok((block_node, pos));
            return Ok((declare_node, pos));
        }
        _ => {
            // try to parse statement
            // let mut block_node = ParseNode::new();
            // block_node.entry = NodeType::Block;

            let (stmt_node, pos) = r#try!(p_stmt(toks, pos));
            return Ok((stmt_node, pos));
            // block_node.child.push(stmt_node);
            //return Ok((block_node, pos));
        }
    }
}
fn p_stmt(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in fn : p_stmt, with pos {}", pos);
    let tok = &toks[pos];
    match tok {
        lexer::TokType::LBrace => {
            // "{" { <block-item> } "}"
            let mut pos = pos + 1;
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Compound);

            // try to get some block item
            while toks[pos] != lexer::TokType::RBrace {
                let (block_node, tmp_pos) = r#try!(p_block(toks, pos));
                stmt_node.child.push(block_node);
                pos = tmp_pos;
            }

            // throw "}"
            pos = pos + 1;
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Ret) => {
            // "return" <exp> ";"
            let pos = pos + 1;
            let (exp_node, mut pos) = r#try!(p_exp(toks, pos));

            let tok = &toks[pos];
            if *tok != lexer::TokType::Semicolon {
                return Err(format!(
                    "Expected `;` in statement, found {:?} at {}",
                    toks[pos], pos
                ));
            }
            pos = pos + 1;

            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Return);
            stmt_node.child.push(exp_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::If) => {
            // "if" "(" <exp> ")" <statement> [ "else" <statement> ]
            // this is the conditional statement
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Conditional("if".to_string()));
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(`"));
            }
            // try to parse exp
            if cfg!(feature = "debug") {
                println!("here pos = {}", pos);
            }
            let pos = pos + 1;
            let (exp_node, pos) = r#try!(p_exp(toks, pos));
            // println!("pos = {}", pos);
            if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                return Err(format!("Missing `)`"));
            }

            let pos = pos + 1;
            // try to parse statement
            if cfg!(feature = "debug") {
                println!("If: parse stmt from pos = {}, tok: {:?}", pos, toks[pos]);
            }
            let (clause_1_node, pos) = r#try!(p_stmt(toks, pos));
            stmt_node.child.push(exp_node);
            stmt_node.child.push(clause_1_node);

            // if has 'else'
            // println!("SHOULD BE HERE , POS = {}", pos);
            if pos < toks.len() && toks[pos] == lexer::TokType::Kwd(lexer::KwdType::Else) {
                // try to parse statement 2
                let pos = pos + 1;
                let (clause_2_node, pos) = r#try!(p_stmt(toks, pos));
                stmt_node.child.push(clause_2_node);
                return Ok((stmt_node, pos));
            } else {
                return Ok((stmt_node, pos));
            }
        }
        lexer::TokType::Kwd(lexer::KwdType::For) => {
            // "for" "(" <exp-option> ";" <exp-option> ";" <exp-option> ")" <statement>
            // "for" "(" <declaration> <exp-option> ";" <exp-option> ")" <statement>
            let mut stmt_node = ParseNode::new();
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(`"));
            }
            let pos = pos + 1;
            // try to parse declaration
            let decl_res = p_declare(toks, pos);
            match decl_res {
                Ok((declare_node, pos)) => {
                    // "for" "(" <declaration> <exp-option> ";" <exp-option> ")" <statement>
                    stmt_node.child.push(declare_node);
                    stmt_node.entry = NodeType::Stmt(StmtType::ForDecl);

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                        return Err(format!("Missing `;` needed by For"));
                    }
                    let pos = pos + 1;

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);
                    if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                        return Err(format!("Missing `)` needed by For"));
                    }
                    let pos = pos + 1;
                    if cfg!(feature = "debug") {
                        println!("pos: {} tok: {:?} before compound layer", pos, toks[pos]);
                    }
                    let mut compound_layer_node = ParseNode::new();
                    compound_layer_node.entry = NodeType::Stmt(StmtType::Compound);
                    let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
                    compound_layer_node.child.push(next_stmt_node);
                    stmt_node.child.push(compound_layer_node);
                    return Ok((stmt_node, pos));
                }
                Err(_) => {
                    // "for" "(" <exp-option> ";" <exp-option> ";" <exp-option> ")" <statement>
                    stmt_node.entry = NodeType::Stmt(StmtType::For);
                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                        return Err(format!("Missing `;` needed by for"));
                    }
                    let pos = pos + 1;

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                        return Err(format!("Missing `;` needed by for"));
                    }
                    let pos = pos + 1;

                    let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));
                    stmt_node.child.push(exp_opt_node);

                    if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                        return Err(format!("Missing `)` needed by for"));
                    }
                    let pos = pos + 1;
                    let mut compound_layer_node = ParseNode::new();
                    let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
                    compound_layer_node.child.push(next_stmt_node);
                    compound_layer_node.entry = NodeType::Stmt(StmtType::Compound);
                    stmt_node.child.push(compound_layer_node);
                    return Ok((stmt_node, pos));
                }
            }
        }
        lexer::TokType::Kwd(lexer::KwdType::While) => {
            // "while" "(" <exp> ")" <statement>
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::While);
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(` needed by While"));
            }

            let pos = pos + 1;
            let (exp_node, pos) = r#try!(p_exp(toks, pos));
            stmt_node.child.push(exp_node);
            if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                return Err(format!("Missing `)`"));
            }
            let pos = pos + 1;

            let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
            stmt_node.child.push(next_stmt_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Do) => {
            // "do" <statement> "while" "(" <exp> ")" ";"
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Do);
            let pos = pos + 1;
            let (next_stmt_node, pos) = r#try!(p_stmt(toks, pos));
            stmt_node.child.push(next_stmt_node);
            // parse while
            if pos >= toks.len() || toks[pos] != lexer::TokType::Kwd(lexer::KwdType::While) {
                return Err(format!("Missing `while` needed by do"));
            }
            let pos = pos + 1;

            if pos >= toks.len() || toks[pos] != lexer::TokType::LParen {
                return Err(format!("Missing `(` needed by do"));
            }
            let pos = pos + 1;

            let (exp_node, pos) = r#try!(p_exp_opt(toks, pos));

            if pos >= toks.len() || toks[pos] != lexer::TokType::RParen {
                return Err(format!("Missing `)` needed by do"));
            }
            let pos = pos + 1;

            if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                return Err(format!("Missing `;` needed by do"));
            }
            let pos = pos + 1;

            stmt_node.child.push(exp_node);
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Continue) => {
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Continue);
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                return Err(format!("Missing `;` needed by continue"));
            }
            let pos = pos + 1;
            return Ok((stmt_node, pos));
        }
        lexer::TokType::Kwd(lexer::KwdType::Break) => {
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Break);
            let pos = pos + 1;
            if pos >= toks.len() || toks[pos] != lexer::TokType::Semicolon {
                return Err(format!("Missing `;` needed by break"));
            }
            let pos = pos + 1;
            return Ok((stmt_node, pos));
        }
        _ => {
            // try to parse exp-option;
            let mut stmt_node = ParseNode::new();
            stmt_node.entry = NodeType::Stmt(StmtType::Exp);
            //let pos = pos + 1;
            let (exp_opt_node, pos) = r#try!(p_exp_opt(toks, pos));

            let tok = &toks[pos];
            if *tok != lexer::TokType::Semicolon {
                return Err(format!("Expected `;`, found {:?} at {}", toks[pos], pos));
            }
            let pos = pos + 1;
            stmt_node.child.push(exp_opt_node);
            return Ok((stmt_node, pos));
        }
    }
}

fn p_factor(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    if cfg!(feature = "debug") {
        println!("in p_factor with pos: {}, tok = {:?}", pos, toks[pos]);
    }
    let mut next = &toks[pos];
    let mut pos = pos + 1;

    match next {
        lexer::TokType::LParen => {
            // parse expression inside parens
            // factor -> exp
            let (exp_node, tmp_pos) = r#try!(p_exp(toks, pos));
            pos = tmp_pos;
            next = &toks[pos];
            pos = pos + 1;
            if *next != lexer::TokType::RParen {
                return Err(format!(
                    "Expected `)` in file:parser.rs, found {:?} at {}",
                    toks[pos], pos
                ));
            }
            let mut factor_node = ParseNode::new();
            factor_node.entry = NodeType::Factor;
            factor_node.child.push(exp_node);
            // println!("out p_factor with pos: {}", pos);
            return Ok((factor_node, pos));
        }
        lexer::TokType::Minus | lexer::TokType::Tilde | lexer::TokType::Exclamation | lexer::TokType::Addr => {
            // factor -> UnExp -> factor
            let mut factor_node = ParseNode::new();
            let mut unexp_node = ParseNode::new();
            factor_node.entry = NodeType::Factor;
            unexp_node.entry = NodeType::UnExp(match next {
                lexer::TokType::Minus => lexer::TokType::Minus,
                lexer::TokType::Tilde => lexer::TokType::Tilde,
                lexer::TokType::Exclamation => lexer::TokType::Exclamation,
                lexer::TokType::Addr => lexer::TokType::Addr,
                _ => panic!("Something strange"),
            });
            let (next_factor_node, pos) = r#try!(p_factor(toks, pos));
            unexp_node.child.push(next_factor_node);
            factor_node.child.push(unexp_node);
            return Ok((factor_node, pos));
        }
        lexer::TokType::String(chars, tag) => {
            let mut string_node = ParseNode::new();
            let mut factor_node = ParseNode::new();
            string_node.entry = NodeType::StringLiteral(chars.to_string(), tag.to_string());
            factor_node.entry = NodeType::Factor;
            factor_node.child.push(string_node);

            return Ok((factor_node, pos));
        }
        lexer::TokType::Literal(n) => {
            // Factor -> Const
            let mut const_node = ParseNode::new();
            let mut factor_node = ParseNode::new();
            const_node.entry = NodeType::Const(*n);
            factor_node.entry = NodeType::Factor;
            factor_node.child.push(const_node);
            // println!("out p_factor with pos: {}", pos);
            return Ok((factor_node, pos));
        }
        lexer::TokType::Identifier(var_name) => {
            if cfg!(feature = "debug") {
                println!("here\n");
            }
            if pos < toks.len() && toks[pos] == lexer::TokType::LParen {
                // Factor -> FnCall
                let mut factor_node = ParseNode::new();
                pos = pos - 1;
                factor_node.entry = NodeType::Factor;
                let (fn_call_node, pos) = r#try!(p_fn_call(toks, pos));
                factor_node.child.push(fn_call_node);
                return Ok((factor_node, pos));
            } else if pos < toks.len() && toks[pos] == lexer::TokType::LBracket {
                // Factor -> Array referencing
                let mut factor_node = ParseNode::new();
                pos = pos - 1;
                factor_node.entry = NodeType::Factor;
                let (arr_ref_node, pos) = r#try!(p_arr_ref(toks, pos));
                factor_node.child.push(arr_ref_node);
                return Ok((factor_node, pos));
            } else {
                // Factor -> Var
                let mut var_node = ParseNode::new();
                let mut factor_node = ParseNode::new();
                var_node.entry = NodeType::Var(var_name.to_string());
                factor_node.entry = NodeType::Factor;
                factor_node.child.push(var_node);
                // println!("out p_factor with pos: {}", pos);
                return Ok((factor_node, pos));
            }
        }
        _ => Err(format!("Factor rule not allowed.")),
    }
}

fn p_arr_ref(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // array reference ::= <id> "[" <exp> "]"
    let mut arr_ref_node = ParseNode::new();
    let mut var_name = String::new();
    match &toks[pos] {
        lexer::TokType::Identifier(name) => {
            var_name = name.to_string();
        }
        _ => {
            return Err(format!(
                "Expected array identifier, foudn {:?} at {}",
                toks[pos], pos
            ));
        }
    }
    arr_ref_node.entry = NodeType::ArrayRef(var_name);

    let mut pos = pos + 1;
    // match '['
    match toks[pos] {
        lexer::TokType::LBracket => {
            pos = pos + 1;
        }
        _ => {
            return Err(format!(
                "Expected `[` needed by array referencing, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }

    let (exp_node, new_pos) = r#try!(p_exp(toks, pos));
    arr_ref_node.child.push(exp_node);
    match toks[new_pos] {
        lexer::TokType::RBracket => {
            pos = new_pos + 1;
        }
        _ => {
            return Err(format!(
                "Expected ']' needed by array referencing, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
    return Ok((arr_ref_node, pos));
}
fn p_fn_call(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // <function-call> ::= id "(" [ <exp> { "," <exp> } ] ")"
    //println!("in fn p_fn_call");
    let mut fn_call_node = ParseNode::new();
    let mut fn_name = String::new();
    match &toks[pos] {
        lexer::TokType::Identifier(name) => {
            fn_name = name.to_string();
        }
        _ => {
            return Err(format!(
                "Expected function name, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
    fn_call_node.entry = NodeType::FnCall(fn_name);
    let mut pos = pos + 1;
    // match '('
    match toks[pos] {
        lexer::TokType::LParen => {
            pos = pos + 1;
        }
        _ => {
            return Err(format!(
                "Expected `(` needed by function call, found {:?} at {}",
                toks[pos], pos
            ));
        }
    }
    while pos < toks.len() && toks[pos] != lexer::TokType::RParen {
        // try to parse argument exp
        let (exp_node, new_pos) = r#try!(p_exp(toks, pos));
        fn_call_node.child.push(exp_node);
        pos = new_pos;

        // match ,
        match &toks[pos] {
            lexer::TokType::Comma => {
                pos = pos + 1;
            }
            lexer::TokType::RParen => {
                continue;
            }
            _ => {
                return Err(format!(
                    "Expected `,` or `)` at the end of exp, found {:?} at {}",
                    toks[pos], pos
                ));
            }
        }
        if toks[pos] == lexer::TokType::RParen {
            break;
        }
    }
    pos = pos + 1;
    return Ok((fn_call_node, pos));
}

fn p_logical_and_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut logAndExp_node = ParseNode::new();
    logAndExp_node.entry = NodeType::LogicalAndExp;

    // LogicalAndExp -> EqualityExp
    let mut pos = pos;
    let (eq_node, tmp_pos) = r#try!(p_eq_exp(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::And {
        logAndExp_node.child.push(eq_node);
        pos = pos - 1;
        return Ok((logAndExp_node, pos));
    }
    // Term -> BinExp -> (EqualityExp, EqualityExp)
    let mut eq_node = eq_node; // change to mutable
    while *tok == lexer::TokType::And {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(lexer::TokType::And);

        let (rhs, tmp_pos) = r#try!(p_eq_exp(toks, pos));

        binexp_node.child.push(eq_node);
        binexp_node.child.push(rhs);
        eq_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    logAndExp_node.child.push(eq_node);
    pos = pos - 1;
    return Ok((logAndExp_node, pos));
}

fn p_eq_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut eq_node = ParseNode::new();
    eq_node.entry = NodeType::EqualityExp;

    let mut pos = pos;
    let (relational_node, tmp_pos) = r#try!(p_relational_exp(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::NotEqual && *tok != lexer::TokType::Equal {
        eq_node.child.push(relational_node);
        pos = pos - 1;
        return Ok((eq_node, pos));
    }

    let mut relational_node = relational_node;
    while *tok == lexer::TokType::Equal || *tok == lexer::TokType::NotEqual {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Equal => lexer::TokType::Equal,
            lexer::TokType::NotEqual => lexer::TokType::NotEqual,
            _ => panic!("in p_eq_exp, something went wrong"),
        });

        let (next_relational_node, tmp_pos) = r#try!(p_relational_exp(toks, pos));

        binexp_node.child.push(relational_node);
        binexp_node.child.push(next_relational_node);
        relational_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    eq_node.child.push(relational_node);
    pos = pos - 1;
    return Ok((eq_node, pos));
}

fn p_relational_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    let mut relational_node = ParseNode::new();
    relational_node.entry = NodeType::RelationalExp;

    let mut pos = pos;
    let (additive_exp_node, tmp_pos) = r#try!(p_additive_exp(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::Lt
        && *tok != lexer::TokType::Gt
        && *tok != lexer::TokType::GreaterEqual
        && *tok != lexer::TokType::LessEqual
    {
        relational_node.child.push(additive_exp_node);
        pos = pos - 1;
        return Ok((relational_node, pos));
    }

    let mut additive_exp_node = additive_exp_node;
    while *tok == lexer::TokType::Lt
        || *tok == lexer::TokType::Gt
        || *tok == lexer::TokType::GreaterEqual
        || *tok == lexer::TokType::LessEqual
    {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Lt => lexer::TokType::Lt,
            lexer::TokType::Gt => lexer::TokType::Gt,
            lexer::TokType::GreaterEqual => lexer::TokType::GreaterEqual,
            lexer::TokType::LessEqual => lexer::TokType::LessEqual,
            _ => panic!("in p_relational_exp, something went wrong"),
        });
        let (next_additive_exp_node, tmp_pos) = r#try!(p_additive_exp(toks, pos));
        binexp_node.child.push(additive_exp_node);
        binexp_node.child.push(next_additive_exp_node);
        additive_exp_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    relational_node.child.push(additive_exp_node);
    pos = pos - 1;
    return Ok((relational_node, pos));
}
fn p_term(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_term with pos: {}", pos);
    let mut term_node = ParseNode::new();
    term_node.entry = NodeType::Term;

    // term -> factor
    let mut pos = pos;
    let (factor_node, tmp_pos) = r#try!(p_factor(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    pos = pos + 1;
    if *tok != lexer::TokType::Multi && *tok != lexer::TokType::Splash {
        term_node.child.push(factor_node);
        pos = pos - 1;
        // println!("1. out p_term with pos: {}", pos);
        return Ok((term_node, pos));
    }

    // term -> BinExp -> (factor_left, factor_right)
    let mut factor_node = factor_node;
    while *tok == lexer::TokType::Multi || *tok == lexer::TokType::Splash {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Multi => lexer::TokType::Multi,
            lexer::TokType::Splash => lexer::TokType::Splash,
            _ => panic!("in p_term, something went wrong"),
        });

        let (next_factor_node, tmp_pos) = r#try!(p_factor(toks, pos));

        binexp_node.child.push(factor_node);
        binexp_node.child.push(next_factor_node);
        factor_node = binexp_node;
        pos = tmp_pos;
        tok = &toks[pos];
        pos = pos + 1;
    }
    term_node.child.push(factor_node);
    pos = pos - 1;
    // println!("2. out p_term with pos: {}", pos);
    return Ok((term_node, pos));
}

fn p_additive_exp(toks: &[lexer::TokType], pos: usize) -> Result<(ParseNode, usize), String> {
    // println!("in p_exp with pos: {}", pos);
    let mut exp_node = ParseNode::new();
    exp_node.entry = NodeType::AdditiveExp;
    // exp -> term
    let mut pos = pos;
    let (term_node, tmp_pos) = r#try!(p_term(toks, pos));
    pos = tmp_pos;
    let mut tok = &toks[pos];
    if *tok != lexer::TokType::Plus && *tok != lexer::TokType::Minus {
        exp_node.child.push(term_node);
        // println!("1.out p_exp with pos: {}", pos);
        return Ok((exp_node, pos));
    }
    // exp -> BinExp()
    //peek next token, if it is lexer::TokType::Plus or lexer::TokType::Minus
    let mut term_node = term_node;
    let mut pos = pos;
    while *tok == lexer::TokType::Plus || *tok == lexer::TokType::Minus {
        let mut binexp_node = ParseNode::new();
        binexp_node.entry = NodeType::BinExp(match tok {
            lexer::TokType::Plus => lexer::TokType::Plus,
            lexer::TokType::Minus => lexer::TokType::Minus,
            _ => panic!("in parser::p_exp, something went wrong"),
        });
        pos = pos + 1;
        let (next_term_node, tmp_pos) = r#try!(p_term(toks, pos));
        pos = tmp_pos;
        binexp_node.child.push(term_node);
        binexp_node.child.push(next_term_node);
        term_node = binexp_node;
        tok = &toks[pos];
    }
    exp_node.child.push(term_node);
    return Ok((exp_node, pos));
}

pub fn parse_prog(input: &str, c_src_name: &str) -> Result<ParseNode, String> {
    let toks = r#try!(lexer::lex(&input));
    let mut prog_node = ParseNode::new();
    prog_node.entry = NodeType::Prog(c_src_name.to_string());
    let mut pos = 0;
    // now we need to add support for global variables
    while pos < toks.len() {
        // try to parse global variable declaration
        let p_res = p_declare(&toks, pos);
        match p_res {
            Ok((decl_node, new_pos)) => {
                pos = new_pos;
                prog_node.child.push(decl_node);
            }
            Err(_) => {
                // try to parse fn definition
                if cfg!(feature = "debug") {
                    println!("try to parse fn definition");
                }
                let (fn_node, new_pos) = r#try!(p_fn(&toks, pos));
                prog_node.child.push(fn_node);
                pos = new_pos;
            }
        }
    }

    return Ok(prog_node);
}

// XXX: should change the return type to Result<String, String> to remove panic!()
pub fn print(tree: &ParseNode, idt: usize) -> String {
    let mut idt_prefix = String::new();
    for _i in 0..idt {
        idt_prefix = idt_prefix + " ";
    }
    match &tree.entry {
        NodeType::StringLiteral(data, tag) => format!(
            "{}n_type: StringLiteral, tag: {}, data: [{}]",
            idt_prefix, tag, data,
        ),
        NodeType::ArrayRef(var_name) => format!(
            "{}n_type: ArrayRef, var_name : {}, [\n{}\n{}]",
            idt_prefix,
            var_name,
            print(tree.child.get(0).unwrap(), idt + 1),
            idt_prefix
        ),
        NodeType::Factor => format!(
            "{}n_type: Factor, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Factor Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::AssignNode(var_name, flag) => {
            match flag {
                false => format!(
                    "{}n_type: AssignNode, Var: {} [\n{}\n{}]",
                    idt_prefix,
                    var_name,
                    print(
                        tree.child.get(0).expect("Assign Node has no child"),
                        idt + 1
                    ),
                    idt_prefix
                ),
                true => {
                    // assign to array
                    format!(
                        "{}n_type: AssignNode array: {} [\n{}\n{}\n{}]",
                        idt_prefix,
                        var_name,
                        print(
                            tree.child
                                .get(0)
                                .expect("Assign to array Node has no index node "),
                            idt + 1
                        ),
                        print(
                            tree.child.get(1).expect("Assign to array node has no rhs"),
                            idt + 1
                        ),
                        idt_prefix,
                    )
                }
            }
        }
        NodeType::BinExp(op) => format!(
            "{}n_type: BinExp, Op: {} [\n{}\n{}\n{}]",
            idt_prefix,
            match op {
                lexer::TokType::Minus => format!("-"),
                lexer::TokType::Plus => format!("+"),
                lexer::TokType::Multi => format!("*"),
                lexer::TokType::Splash => format!("/"),
                lexer::TokType::And => format!("&&"),
                lexer::TokType::Or => format!("||"),
                lexer::TokType::Equal => format!("=="),
                lexer::TokType::NotEqual => format!("!="),
                lexer::TokType::GreaterEqual => format!(">="),
                lexer::TokType::LessEqual => format!("<="),
                lexer::TokType::Lt => format!("<"),
                lexer::TokType::Gt => format!(">"),
                _ => panic!(format!(
                    "Operator `{:?}` for Binary Expression not supported",
                    &op
                )),
            },
            print(tree.child.get(0).expect("BinExp no lhs"), idt + 1),
            print(tree.child.get(1).expect("BinExp no rhs"), idt + 1),
            idt_prefix
        ),
        NodeType::Term => format!(
            "{}n_type: Term, [\n{}\n{}]",
            idt_prefix,
            print(tree.child.get(0).expect("Term Node has no child"), idt + 1),
            idt_prefix
        ),
        NodeType::Prog(prog_name) => {
            let mut prog_body = String::new();
            for it in tree.child.iter() {
                prog_body.push_str(&print(it, idt + 1));
                prog_body.push_str("\n");
            }
            format!(
                "{}n_type: Prog, Name:{} [\n{}\n{}]",
                idt_prefix, prog_name, prog_body, idt_prefix
            )
        }
        NodeType::FnCall(fn_name) => {
            let mut tmp = String::new();
            let mut inc = 0;
            for it in tree.child.iter() {
                if inc > 0 {
                    tmp.push_str("\n");
                }
                tmp.push_str(&print(it, idt + 1));
                inc = inc + 1;
            }

            format!(
                "{}n_type: FnCall, Name: {} exp_list: [\n{}\n{}]",
                idt_prefix, fn_name, tmp, idt_prefix
            )
            // list of exp
        }
        NodeType::Fn(fn_name, vars) => {
            let mut tmp = String::new();
            let mut inc = 0;
            for it in tree.child.iter() {
                if inc > 0 {
                    tmp.push_str("\n");
                }
                tmp.push_str(&print(it, idt + 1));
                inc = inc + 1;
            }
            let mut var_list_string = String::new();
            match vars {
                Some(var_list) => {
                    for var in var_list {
                        var_list_string.push_str(" ");
                        var_list_string.push_str(var);
                        var_list_string.push_str(" ");
                    }
                }
                None => {}
            }
            format!(
                "{}n_type: Fn, Name: {} var_list: [{}]\n\
                 {}[\n{}\n{}]",
                idt_prefix, fn_name, var_list_string, idt_prefix, tmp, idt_prefix
            )
        }
        NodeType::Declare(var_name, t) => match t {
            DataType::I64 => {
                if tree.child.is_empty() {
                    format!(
                        "{}n_type: Declare, type: Int var_name: {}",
                        idt_prefix, var_name
                    )
                } else {
                    format!(
                        "{}n_type: Declare, type: Int var_name: {}, [\n{}\n{}]",
                        idt_prefix,
                        var_name,
                        print(
                            tree.child.get(0).expect("Declare Node has no child"),
                            idt + 1
                        ),
                        idt_prefix
                    )
                }
            }
            DataType::Arr64(len) => format!(
                "{}n_type: Declare, type: Array  var_name: {}, length: {}",
                idt_prefix, var_name, len,
            ),
        },
        NodeType::ConditionalExp => {
            let mut tmp = String::new();
            let mut inc = 0;
            for it in tree.child.iter() {
                if inc > 0 {
                    tmp.push_str("\n");
                }
                tmp.push_str(&print(it, idt + 1));
                inc = inc + 1;
            }
            format!(
                "{}n_type: Stmt::ConditionalExp, [\n{}\n{}]",
                idt_prefix, tmp, idt_prefix
            )
        }
        NodeType::Stmt(stmt) => match stmt {
            StmtType::For => {
                let exp_opt_1 = print(tree.child.get(0).expect("No exp1 in for"), idt + 1);
                let exp_opt_2 = print(tree.child.get(1).expect("No exp2 in for"), idt + 1);
                let exp_opt_3 = print(tree.child.get(2).expect("No exp3 in for"), idt + 1);
                let stmt = print(tree.child.get(3).expect("No stmt in for"), idt + 1);
                format!(
                    "{}n_type: Stmt:ForDeclare, [\n\
                     {}declare: [\n{}\n{}]\n\
                     {}exp1:    [\n{}\n{}]\n\
                     {}exp2:    [\n{}\n{}]\n\
                     {}stmt:    [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    exp_opt_1,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_2,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_3,
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::ForDecl => {
                let d = print(tree.child.get(0).expect("No declaration in for"), idt + 1);
                let exp_opt_1 = print(tree.child.get(1).expect("No exp1 in for"), idt + 1);
                let exp_opt_2 = print(tree.child.get(2).expect("No exp2 in for"), idt + 1);
                let stmt = print(tree.child.get(3).expect("No stmt in for"), idt + 1);

                format!(
                    "{}n_type: Stmt:ForDeclare, [\n\
                     {}declare: [\n{}\n{}]\n\
                     {}exp1:    [\n{}\n{}]\n\
                     {}exp2:    [\n{}\n{}]\n\
                     {}stmt:    [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    d,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_1,
                    idt_prefix,
                    idt_prefix,
                    exp_opt_2,
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::Do => {
                let stmt = print(tree.child.get(0).expect("No stmt in do"), idt + 1);
                let exp = print(tree.child.get(1).expect("No exp in do"), idt + 1);
                format!(
                    "{}n_type: Stmt:Do, [\n\
                     {}           stmt: [\n{}\n{}]\n\
                     {}           exp:  [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix,
                    exp,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::While => {
                let exp = print(tree.child.get(0).expect("No stmt in do"), idt + 1);
                let stmt = print(tree.child.get(1).expect("No exp in do"), idt + 1);
                format!(
                    "{}n_type: Stmt:While, [\n\
                     {}               exp: [\n{}\n{}]\n\
                     {}              stmt:  [\n{}\n{}]\n\
                     {}]",
                    idt_prefix,
                    idt_prefix,
                    exp,
                    idt_prefix,
                    idt_prefix,
                    stmt,
                    idt_prefix,
                    idt_prefix
                )
            }
            StmtType::Continue => format!("{}n_type: Continue", idt_prefix),
            StmtType::Break => format!("{}n_type: Break", idt_prefix),
            StmtType::Return => format!(
                "{}n_type: Stmt::Return, [\n{}\n{}]",
                idt_prefix,
                print(
                    tree.child
                        .get(0)
                        .expect("Statement::Return Node has no child"),
                    idt + 1
                ),
                idt_prefix
            ),
            StmtType::Exp => format!(
                "{}n_type: Stmt::Exp, [\n{}\n{}]",
                idt_prefix,
                print(
                    tree.child.get(0).expect("Statement::Exp Node has no child"),
                    idt + 1
                ),
                idt_prefix
            ),
            StmtType::Conditional(op) => {
                let mut tmp = String::new();
                let mut inc = 0;
                for it in tree.child.iter() {
                    if inc > 0 {
                        tmp.push_str("\n");
                    }
                    tmp.push_str(&print(it, idt + 1));
                    inc = inc + 1;
                }
                format!(
                    "{}n_type: Stmt::Conditional, Op: {} [\n{}\n{}]",
                    idt_prefix, op, tmp, idt_prefix
                )
            }
            StmtType::Compound => {
                let mut tmp = String::new();
                let mut inc = 0;

                for it in tree.child.iter() {
                    if inc > 0 {
                        tmp.push_str("\n");
                    }
                    tmp.push_str(&print(it, idt + 1));
                    inc += 1;
                }
                format!(
                    "{}n_type: Stmt::Compound, [\n{}\n{}]",
                    idt_prefix, tmp, idt_prefix
                )
            }
        },
        NodeType::LogicalOrExp |
        NodeType::LogicalAndExp|
        NodeType::EqualityExp  |
        NodeType::RelationalExp|
        NodeType::AdditiveExp =>
            print(tree.child.get(0).expect(&format!("{:?} Node has no child", tree.entry)), idt),

        NodeType::ExpOption => match tree.child.len() {
            0 => format!("{}n_type: ExpOption", idt_prefix),
            1 => format!(
                "{}n_type: ExpOption, [\n{}\n{}]",
                idt_prefix,
                print(tree.child.get(0).expect("ExpOption has no child"), idt + 1),
                idt_prefix
            ),
            _ => panic!(format!(
                "ExpOption can only have 0 or 1 child node, but found {} child node",
                tree.child.len()
            )),
        },
        NodeType::Exp => format!(
            "{}n_type: Exp, [\n{}\n{}]",
            idt_prefix,
            print(
                tree.child.get(0).expect("Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Block => format!(
            "{}n_type: Block, [\n{}\n{}]",
            idt_prefix,
            print(tree.child.get(0).expect("Block node has no child"), idt + 1),
            idt_prefix
        ),
        NodeType::UnExp(Op) => format!(
            "{}n_type: UnExp, Op: {}, [\n{}\n{}]",
            idt_prefix,
            match Op {
                lexer::TokType::Minus => "-".to_string(),
                lexer::TokType::Tilde => "~".to_string(),
                lexer::TokType::Exclamation => "!".to_string(),
                lexer::TokType::Addr => "&".to_string(),
                _ => panic!("Operator for Unary Expression not supported"),
            },
            print(
                tree.child
                    .get(0)
                    .expect("Unary Expression Node has no child"),
                idt + 1
            ),
            idt_prefix
        ),
        NodeType::Var(var_name) => format!("{}n_type, Variable, Name : {}", idt_prefix, var_name),
        NodeType::Const(n) => format!("{}n_type: Const, Value: {}", idt_prefix, n),
        _ => panic!(format!(
            "in parser::print, {:?} Node type not implemented",
            &tree.entry
        )),
    }
}
