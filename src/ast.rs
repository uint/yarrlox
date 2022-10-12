// In the book, Robert suggests the visitor pattern as a way to separate behavior
// from the AST definition. This probably makes sense for Java or C++.
//
// In Rust, we can use pattern matching and algebraic data types instead. It's possible
// to implement the visitor pattern, but that means more boilerplate and less
// idiomatic code. I didn't find enough justification for using the visitor pattern.

yarrlox_macros::define_ast! {
    enum Expr<'src> {
        struct Binary<'src> {
            left: Box<Expr<'src>>,
            operator: BinaryOp,
            right: Box<Expr<'src>>,
        }
        struct Grouping<'src> {
            expr: Box<Expr<'src>>,
        }
        enum Literal<'src> {
            struct StringLit<'src>(&'src str);
            struct NumLit<'src>(&'src str);
        }
        struct Unary<'src> {
            operator: BinaryOp,
            right: Box<Expr<'src>>,
        }
    }
}

pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
