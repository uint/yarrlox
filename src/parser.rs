use crate::ast::{Binary, BinaryOp, Expr};
use crate::lexer::{Lexer, Token};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    fn parse_expr(&mut self) -> Expr<'src> {
        self.parse_binop(0)
    }

    fn parse_binop(&mut self, min_prec: u32) -> Expr<'src> {
        let mut expr = self.parse_unary();

        while let Some(op) = self.peek_binary_operator() {
            let (prec, assoc) = op.prec_assoc();

            if prec < min_prec {
                break;
            }

            // consume the operator token
            self.lexer.next();

            let next_min_prec = match assoc {
                Assoc::Left => prec + 1,
                Assoc::Right => prec,
            };

            let rhs = self.parse_binop(next_min_prec);

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                op,
                right: Box::new(rhs),
            });
        }

        expr
    }

    fn parse_unary(&mut self) -> Expr<'src> {
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Expr<'src> {
        todo!()
    }

    fn peek_binary_operator(&mut self) -> Option<BinaryOp> {
        match self.lexer.peek() {
            Some(Token::Plus) => Some(BinaryOp::Add),
            Some(Token::Minus) => Some(BinaryOp::Sub),
            Some(Token::Star) => Some(BinaryOp::Mul),
            Some(Token::Slash) => Some(BinaryOp::Div),
            _ => None,
        }
    }
}

enum Assoc {
    Left,
    Right,
}

trait Precedence {
    fn prec_assoc(&self) -> (u32, Assoc);
}

impl Precedence for BinaryOp {
    fn prec_assoc(&self) -> (u32, Assoc) {
        match self {
            BinaryOp::Eq => (0, Assoc::Left),
            BinaryOp::NotEq => (0, Assoc::Left),
            BinaryOp::Gt => (1, Assoc::Left),
            BinaryOp::Lt => (1, Assoc::Left),
            BinaryOp::Gte => (1, Assoc::Left),
            BinaryOp::Lte => (1, Assoc::Left),
            BinaryOp::Add => (2, Assoc::Left),
            BinaryOp::Sub => (2, Assoc::Left),
            BinaryOp::Mul => (3, Assoc::Left),
            BinaryOp::Div => (3, Assoc::Left),
        }
    }
}
