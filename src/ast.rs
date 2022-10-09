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
