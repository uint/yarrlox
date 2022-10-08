use enum_dispatch::enum_dispatch;

#[enum_dispatch]
enum Expr {
    Binary,
}

#[enum_dispatch(Expr)]
trait Expression: private::Seal {}

struct Binary {
    left: Box<Expr>,
    operator: BinaryOp,
    right: Box<Expr>,
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Expression for Binary {}

mod private {
    pub(super) trait Seal {}

    impl Seal for super::Expr {}
    impl Seal for super::Binary {}
}
