//! Abstract syntax tree for Mycor.

use myc_span::Span;

pub type Ident = String;

#[derive(Clone, Debug)]
pub struct File {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Item {
    Fn(FnDef),
    Struct(StructDef),
    Enum(EnumDef),
    Const(ConstDef),
    Static(StaticDef),
    TypeAlias(TypeAlias),
    Mod(ModDef),
    Use(UseDef),
    Impl(ImplDef),
    Trait(TraitDef),
}

#[derive(Clone, Debug)]
pub struct FnDef {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub body: Option<Block>,
    pub is_unsafe: bool,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct GenericParam {
    pub name: Ident,
    pub bounds: Vec<Type>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: Option<Type>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDef {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumDef {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<Variant>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: Ident,
    pub fields: Vec<Type>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ConstDef {
    pub name: Ident,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StaticDef {
    pub name: Ident,
    pub ty: Option<Type>,
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeAlias {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ModDef {
    pub name: Ident,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct UseDef {
    pub path: Vec<Ident>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImplDef {
    pub trait_name: Option<Ident>,
    pub for_type: Type,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TraitDef {
    pub name: Ident,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Item(Item),
    Defer(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}

#[derive(Clone, Debug)]
pub struct LetStmt {
    pub name: Ident,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Lit, Span),
    Path(Vec<Ident>, Span),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: Ident,
        args: Vec<Expr>,
        span: Span,
    },
    Field {
        base: Box<Expr>,
        name: Ident,
        span: Span,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },
    While {
        cond: Box<Expr>,
        body: Block,
        span: Span,
    },
    For {
        name: Ident,
        iter: Box<Expr>,
        body: Block,
        span: Span,
    },
    Loop {
        body: Block,
        span: Span,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Block(Block),
    Closure {
        params: Vec<Ident>,
        body: Box<Expr>,
        span: Span,
    },
    Cast {
        expr: Box<Expr>,
        ty: Type,
        span: Span,
    },
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },
    Try {
        expr: Box<Expr>,
        span: Span,
    },
    Unsafe {
        block: Block,
        span: Span,
    },
    Tuple {
        elems: Vec<Expr>,
        span: Span,
    },
    Array {
        elems: Vec<Expr>,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Range,
    RangeIncl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Deref,
    Ref,
    RefMut,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard(Span),
    Binding(Ident, Span),
    Lit(Lit, Span),
    Tuple(Vec<Pattern>, Span),
    Variant(Ident, Vec<Pattern>, Span),
    Or(Vec<Pattern>, Span),
}

#[derive(Clone, Debug)]
pub enum Type {
    Named(Ident, Vec<Type>, Span),
    Ref(Box<Type>, Span),
    RefMut(Box<Type>, Span),
    Ptr(Box<Type>, Span),
    PtrMut(Box<Type>, Span),
    Array(Box<Type>, Option<Box<Expr>>, Span),
    Slice(Box<Type>, Span),
    Tuple(Vec<Type>, Span),
    FnPtr(Vec<Type>, Box<Type>, Span),
    Infer(Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Named(_, _, s) => *s,
            Type::Ref(_, s) => *s,
            Type::RefMut(_, s) => *s,
            Type::Ptr(_, s) => *s,
            Type::PtrMut(_, s) => *s,
            Type::Array(_, _, s) => *s,
            Type::Slice(_, s) => *s,
            Type::Tuple(_, s) => *s,
            Type::FnPtr(_, _, s) => *s,
            Type::Infer(s) => *s,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Attr {
    pub name: Ident,
    pub span: Span,
}
