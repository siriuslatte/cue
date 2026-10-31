use crate::{
    Span,
    ast::{Binary, Unary},
    types::Type,
};

pub(crate) struct Program {
    pub scenes: Vec<Scene>,
}

pub(crate) struct Scene {
    pub name: String,
    pub locals: usize,
    pub body: Vec<Stmt>,
}

pub(crate) struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

pub(crate) enum StmtKind {
    Dialogue(String, Expr),
    Bind(usize, Expr),
    Assign {
        local: usize,
        path: Vec<Access>,
        value: Expr,
    },
    If {
        condition: Expr,
        yes: Vec<Stmt>,
        no: Vec<Stmt>,
    },
    For {
        local: usize,
        collection: Expr,
        body: Vec<Stmt>,
    },
    Match {
        value: Expr,
        arms: Vec<Arm>,
    },
    Block(Vec<Stmt>),
}

pub(crate) struct Arm {
    pub variant: String,
    pub binding: Option<usize>,
    pub body: Vec<Stmt>,
}

pub(crate) enum Access {
    Field(String),
    Index(Expr),
}

pub(crate) struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}

pub(crate) enum ExprKind {
    Number(f64),
    Bool(bool),
    Text(String),
    None,
    Local(usize),
    List(Vec<Expr>),
    Record(Vec<(String, Expr)>),
    Enum {
        enumeration: String,
        variant: String,
        payload: Option<Box<Expr>>,
    },
    Unary(Unary, Box<Expr>),
    Binary(Binary, Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Length(Box<Expr>),
    String(Box<Expr>),
}
