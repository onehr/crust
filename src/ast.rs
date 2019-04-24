use crate::lexer;
use crate::symtable::TypeExpression;
// ----------------------------------------------------------------------------------------
#[derive(PartialEq, Clone, Debug)]
pub enum NodeType {
    BinaryExpression(lexer::TokType),
    PrimaryExpression,
    Constant(ConstantType),
    EnumerationConstant(String),
    Identifier(String),
    STRING(String),
    GenericSelection,
    GenericAssociation,
    GenericAssocList,
    PostfixExpression,
    PostfixExpressionPost(lexer::TokType),
    ArgumentExpressionList,
    UnaryExpression(Option<lexer::TokType>),
    UnaryOperator(lexer::TokType),
    CastExpression,
    MultiplicativeExpression,
    AdditiveExpression,
    ShiftExpression,
    RelationalExpression,
    EqualityExpression,
    AndExpression,
    ExclusiveOrExpression,
    InclusiveOrExpression,
    LogicalAndExpression,
    LogicalOrExpression,
    ConditionalExpression,
    AssignmentExpression,
    AssignmentOperator(lexer::TokType),
    Expression,
    ConstantExpression,
    Declaration,
    DeclarationSpecifiers,
    InitDeclaratorList,
    InitDeclarator,
    StorageClassSpecifier(lexer::TokType),
    TypeSpecifier(Option<lexer::TokType>),
    StructOrUnionSpecifier,
    StructOrUnion(lexer::TokType),
    StructDeclarationList,
    StructDeclaration,
    SpecifierQualifier,
    StructDeclaratorList,
    StructDeclarator,
    EnumSpecifier(Option<String>), // Option<Identifer>
    EnumeratorList,
    Enumerator,
    AtomicTypeSpecifier,
    TypeQualifier(lexer::TokType),
    FunctionSpecifier(lexer::TokType),
    AlignmentSpecifier,
    Declarator,
    DirectDeclarator,
    DirectDeclaratorPostList,
    DirectDeclaratorPost(lexer::TokType),
    Pointer, // one node represents one `*`
    TypeQualifierList,
    ParameterDeclaration,
    ParameterTypeList(bool), // true: has ..., var_arg_list
    ParameterList,
    IdentifierList,
    TypeName,
    AbstractDeclarator,
    InitializerList,
    DirectAbstractDeclarator,
    DirectAbstractDeclaratorBlock(lexer::TokType),
    Initializer,
    Designation,
    DesignatorList,
    Designator,
    StaticAssertDeclaration,
    Statement,
    LabeledStatement(String), // string: label
    CompoundStatement,
    BlockItemList,
    BlockItem,
    ExpressionStatement,
    SelectionStatement(lexer::TokType), // if, switch
    IterationStatement(lexer::TokType),
    JumpStatement(String, Option<String>), // String: goto, continue, ... Option<String> : label
    TranslationUnit,
    ExternalDeclaration,
    FunctionDefinition,
    DeclarationList,
}
#[derive(PartialEq, Clone, Debug)]
pub enum ConstantType {
    I64(i64),
    F64(f64),
    String(String),
}

#[derive(PartialEq, Clone, Debug)]
pub struct ParseNode {
    pub child: Vec<ParseNode>,
    pub entry: NodeType,
    pub type_exp: TypeExpression,
}

impl ParseNode {
    pub fn new(s: NodeType) -> ParseNode {
        ParseNode {
            child: Vec::new(),
            entry: s,
            type_exp: TypeExpression::new(),
        }
    }
}
