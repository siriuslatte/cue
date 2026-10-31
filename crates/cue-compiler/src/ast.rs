use crate::Span;

#[derive(Debug, Clone)]
pub(crate) struct Name {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Default)]
pub(crate) struct Program {
    pub actors: Vec<Actor>,
    pub enums: Vec<Enum>,
    pub scenes: Vec<Scene>,
}

#[derive(Debug)]
pub(crate) struct Actor {
    pub name: Name,
    pub id: String,
}

#[derive(Debug)]
pub(crate) struct Enum {
    pub name: Name,
    pub variants: Vec<(Name, Option<TypeExpr>)>,
}

#[derive(Debug)]
pub(crate) struct Scene {
    pub name: Name,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub(crate) struct TypeExpr {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug)]
pub(crate) enum TypeKind {
    Name(String),
    List(Box<TypeExpr>),
    Optional(Box<TypeExpr>),
    Record(Vec<(Name, TypeExpr)>),
}

#[derive(Debug)]
pub(crate) struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug)]
pub(crate) enum StmtKind {
    Dialogue(Name, Expr),
    Bind {
        name: Name,
        mutable: bool,
        annotation: Option<TypeExpr>,
        value: Expr,
    },
    Assign(Expr, Expr),
    If {
        condition: Expr,
        yes: Vec<Stmt>,
        no: Vec<Stmt>,
    },
    For {
        name: Name,
        collection: Expr,
        body: Vec<Stmt>,
    },
    Match {
        value: Expr,
        arms: Vec<Arm>,
    },
    Block(Vec<Stmt>),
}

#[derive(Debug)]
pub(crate) struct Arm {
    pub enumeration: Name,
    pub variant: Name,
    pub binding: Option<Name>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub(crate) struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug)]
pub(crate) enum ExprKind {
    Number(f64),
    Bool(bool),
    Text(String),
    None,
    Name(Name),
    List(Vec<Expr>),
    Record(Vec<(Name, Expr)>),
    Unary(Unary, Box<Expr>),
    Binary(Binary, Box<Expr>, Box<Expr>),
    Field(Box<Expr>, Name),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

impl Expr {
    pub fn depth(&self) -> usize {
        1 + match &self.kind {
            ExprKind::Unary(_, value) | ExprKind::Field(value, _) => value.depth(),
            ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => a.depth().max(b.depth()),
            ExprKind::Call(callee, args) => callee
                .depth()
                .max(args.iter().map(Self::depth).max().unwrap_or(0)),
            ExprKind::List(items) => items.iter().map(Self::depth).max().unwrap_or(0),
            ExprKind::Record(fields) => fields
                .iter()
                .map(|(_, value)| value.depth())
                .max()
                .unwrap_or(0),
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Unary {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Binary {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Power,
    Concat,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Coalesce,
}

impl Binary {
    pub fn comparison(self) -> bool {
        matches!(
            self,
            Self::Less
                | Self::LessEqual
                | Self::Greater
                | Self::GreaterEqual
                | Self::Equal
                | Self::NotEqual
        )
    }
}
