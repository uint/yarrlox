use std::collections::VecDeque;

use crate::ast::{Binary, BinaryOp, Expr, Identifier, Literal, NumLit, StringLit, Unary, UnaryOp};
use crate::lexer::{Lexer, Token};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            lexer: Lexer::new(src),
        }
    }

    pub fn parse_expr(&mut self) -> Expr<'src> {
        self.parse_binop(0)
    }

    /// Parses binary operations using precedence climbing. This is conceptually the
    /// same thing as the book describes when parsing equality, comparisons, terms,
    /// and factors, but here we condense all of that into one step.
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

    /// A trivial unary expression parser. It simply applies unary operators right-to-left
    fn parse_unary(&mut self) -> Expr<'src> {
        let mut ops = VecDeque::new();

        while let Some(op) = self.peek_unary_operator() {
            self.lexer.next();
            ops.push_front(op);
        }

        let mut expr = self.parse_atom();

        for op in ops.into_iter() {
            expr = Expr::Unary(Unary {
                op,
                right: Box::new(expr),
            });
        }

        expr
    }

    /// Parses literals and groupings (parenthesized expressions)
    fn parse_atom(&mut self) -> Expr<'src> {
        let (token, _span) = self.lexer.next().unwrap();
        match token {
            Token::NumLit(l) => Expr::Literal(Literal::NumLit(NumLit(l))),
            Token::StringLit(l) => Expr::Literal(Literal::StringLit(StringLit(l))),
            Token::Identifier(l) => Expr::Literal(Literal::Identifier(Identifier(l))),
            Token::LeftParen => self.parse_paren_expr(),
            _ => panic!("unexpected token woes!"),
        }
    }

    fn parse_paren_expr(&mut self) -> Expr<'src> {
        let expr = self.parse_expr();
        self.parse_token(Token::RightParen);
        expr
    }

    fn parse_token(&mut self, expected: Token) {
        let (token, _span) = self.lexer.next().unwrap();
        assert_eq!(token, expected);
    }

    fn peek_binary_operator(&mut self) -> Option<BinaryOp> {
        match self.lexer.peek() {
            Some(Token::Plus) => Some(BinaryOp::Add),
            Some(Token::Minus) => Some(BinaryOp::Sub),
            Some(Token::Star) => Some(BinaryOp::Mul),
            Some(Token::Slash) => Some(BinaryOp::Div),
            Some(Token::Greater) => Some(BinaryOp::Gt),
            Some(Token::GreaterEqual) => Some(BinaryOp::Gte),
            Some(Token::Less) => Some(BinaryOp::Lt),
            Some(Token::LessEqual) => Some(BinaryOp::Lte),
            Some(Token::EqualEqual) => Some(BinaryOp::Eq),
            Some(Token::BangEqual) => Some(BinaryOp::NotEq),
            _ => None,
        }
    }

    fn peek_unary_operator(&mut self) -> Option<UnaryOp> {
        match self.lexer.peek() {
            Some(Token::Minus) => Some(UnaryOp::Negation),
            Some(Token::Bang) => Some(UnaryOp::Not),
            _ => None,
        }
    }
}

#[allow(unused)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expr_with_precedence() {
        let expr = Parser::new("1 >= 2 * 3 + 4").parse_expr();
        dbg!(&expr);

        let expected = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal::NumLit(NumLit("1")))),
            right: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Literal(Literal::NumLit(NumLit("2")))),
                    right: Box::new(Expr::Literal(Literal::NumLit(NumLit("3")))),
                    op: BinaryOp::Mul,
                })),
                right: Box::new(Expr::Literal(Literal::NumLit(NumLit("4")))),
                op: BinaryOp::Add,
            })),
            op: BinaryOp::Gte,
        });

        assert_eq!(expr, expected);
    }
}
