// In the book, Robert suggests the visitor pattern as a way to separate behavior
// from the AST definition. This probably makes sense for Java or C++.
//
// In Rust, we can use pattern matching and algebraic data types instead. It's possible
// to implement the visitor pattern, but that means more boilerplate and less
// idiomatic code. I didn't find enough justification for using the visitor pattern.

use std::ops::Range;

structstruck::strike! {
    #[strikethrough[derive(Clone, Debug, PartialEq, Eq)]]
    pub enum Expr {
        Assign(pub struct {
            pub name: Reference,
            pub value: Box<Expr>,
        }),
        Binary(pub struct {
             pub left: Box<Expr>,
             pub op: pub enum BinaryOp {
                 LogicOr,
                 LogicAnd,
                 Add,
                 Sub,
                 Mul,
                 Div,
                 Gt,
                 Lt,
                 Gte,
                 Lte,
                 Eq,
                 NotEq,
             },
             pub right: Box<Expr>,
        }),
        Literal(pub enum {
            StringLit(pub struct(pub String)),
            NumLit(pub struct(pub String)),
            Identifier(Reference),
            Nil,
            Bool(bool),
        }),
        Unary(pub struct {
            pub op: pub enum UnaryOp {
                Negation,
                Not,
            },
            pub right: Box<Expr>,
        }),
        Grouping(pub struct {
            pub expr: Box<Expr>,
        }),
        Call(pub struct {
            pub callee: Box<Expr>,
            pub paren: Range<usize>,
            pub args: Vec<Expr>,
        }),
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Reference {
    pub id: usize,
    pub ident: String,
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

structstruck::strike! {
    #[strikethrough[derive(Clone, Debug, PartialEq)]]
    pub enum Stmt {
        Block(Vec<Stmt>),
        Expr(Expr),
        Function (pub struct {
            pub name: String,
            pub params: Vec<String>,
            pub body: Vec<Stmt>,
        }),
        If {
            condition: Expr,
            then_branch: Box<Stmt>,
            else_branch: Option<Box<Stmt>>,
        },
        Print(Expr),
        Return(Option<Expr>),
        Var {
            name: String,
            initializer: Option<Expr>,
        },
        While {
            condition: Expr,
            body: Box<Stmt>,
        },
        Break,
    }
}
