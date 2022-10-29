// In the book, Robert suggests the visitor pattern as a way to separate behavior
// from the AST definition. This probably makes sense for Java or C++.
//
// In Rust, we can use pattern matching and algebraic data types instead. It's possible
// to implement the visitor pattern, but that means more boilerplate and less
// idiomatic code. I didn't find enough justification for using the visitor pattern.

structstruck::strike! {
    #[strikethrough[derive(Clone, Debug, PartialEq)]]
    pub enum Expr<'src> {
        Assign(pub struct<'src> {
            pub name: Identifier<'src>,
            pub value: Box<Expr<'src>>,
        }),
        Binary(pub struct<'src> {
             pub left: Box<Expr<'src>>,
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
             pub right: Box<Expr<'src>>,
        }),
        Literal(pub enum<'src> {
            StringLit(pub struct<'src>(pub &'src str)),
            NumLit(pub struct<'src>(pub &'src str)),
            Identifier(pub struct<'src>(pub &'src str)),
            Nil,
            Bool(bool),
        }),
        Unary(pub struct<'src> {
            pub op: pub enum UnaryOp {
                Negation,
                Not,
            },
            pub right: Box<Expr<'src>>,
        }),
        Grouping(pub struct<'src> {
            pub expr: Box<Expr<'src>>,
        }),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt<'src> {
    Block(Vec<Stmt<'src>>),
    Expr(Expr<'src>),
    If {
        condition: Expr<'src>,
        then_branch: Box<Stmt<'src>>,
        else_branch: Option<Box<Stmt<'src>>>,
    },
    Print(Expr<'src>),
    Var {
        name: Identifier<'src>,
        initializer: Option<Expr<'src>>,
    },
}
