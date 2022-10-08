use enum_dispatch::enum_dispatch;

/// This reduces boilerplate for implementing all types of expression. It provides an `Expr`
/// enum for all possible types of expression, a sealed `Expression` trait that both `Expr`
/// and expression "subtypes" implement, and expression "subtype" definitions.
macro_rules! define_exprs {
    ($($ident:ident $fields:tt),*) => {
        #[enum_dispatch]
        enum Expr {
            $($ident),*
        }

        #[enum_dispatch(Expr)]
        trait Expression: private::Seal {}

        $(struct $ident $fields)*

        $(impl Expression for $ident {})*

        mod private {
            /// This prevents `Expression` from being implemented outside of this crate.
            /// Also see: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
            pub(super) trait Seal {}

            impl Seal for super::Expr {}
            $(impl Seal for super::$ident {})*
        }
    };
}

define_exprs! {
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {},
    Unary {}
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
