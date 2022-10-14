// In the book, Robert suggests the visitor pattern as a way to separate behavior
// from the AST definition. This probably makes sense for Java or C++.
//
// In Rust, we can use pattern matching and algebraic data types instead. It's possible
// to implement the visitor pattern, but that means more boilerplate and less
// idiomatic code. I didn't find enough justification for using the visitor pattern.

yarrlox_macros::define_ast! {
    enum Expr<'src> {
        struct Binary<'src> {
            pub left: Box<Expr<'src>>,
            pub op: BinaryOp,
            pub right: Box<Expr<'src>>,
        }
        struct Grouping<'src> {
            pub expr: Box<Expr<'src>>,
        }
        enum Literal<'src> {
            struct StringLit<'src>(pub &'src str);
            struct NumLit<'src>(pub &'src str);
            struct Identifier<'src>(pub &'src str);
        }
        struct Unary<'src> {
            pub op: UnaryOp,
            pub right: Box<Expr<'src>>,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
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
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Negation,
    Not,
}
