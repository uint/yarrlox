// In the book, Robert suggests the visitor pattern as a way to separate behavior
// from the AST definition. This probably makes sense for Java or C++.
//
// In Rust, we can use pattern matching and algebraic data types instead. It's possible
// to implement the visitor pattern, but that means more boilerplate and less
// idiomatic code. I didn't find enough justification for using the visitor pattern.

structstruck::strike! {
    #[strikethrough[derive(Clone, Debug, PartialEq)]]
    pub enum Expr<'src> {
        Binary(pub struct<'src> {
             pub left: Box<Expr<'src>>,
             pub op: pub enum BinaryOp {
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
