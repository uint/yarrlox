use std::collections::VecDeque;
use std::string::ParseError;

use crate::ast::{
    Binary, BinaryOp, Expr, Grouping, Identifier, Literal, NumLit, Stmt, StringLit, Unary, UnaryOp,
};
use crate::errors::Error;
use crate::lexer::Lexer;
use crate::token::{SpannedToken, Token};

pub struct Parser<'src> {
    errors: Vec<ParserError<'src>>,
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            errors: vec![],
            lexer: Lexer::new(src),
        }
    }

    pub fn errors(&self) -> &[ParserError<'src>] {
        &self.errors
    }

    fn synchronize(&mut self) {
        if let Some(mut previous) = self.lexer.next() {
            while let Some(token) = self.lexer.peek() {
                use Token::*;

                if previous.token == Semicolon {
                    return;
                }

                match token {
                    Class | Fun | Var | For | If | While | Print | Return => return,
                    _ => previous = self.lexer.next().unwrap(),
                }
            }
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.lexer.peek().is_none()
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt<'src>>, Vec<ParserError>> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.parse_stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => errors.push(err),
            }
        }

        if errors.is_empty() {
            Ok(stmts)
        } else {
            Err(errors)
        }
    }

    fn parse_stmt(&mut self) -> ParseResult<'src, Stmt<'src>> {
        let stmt = match self.lexer.peek().unwrap() {
            Token::Print => self.parse_print_stmt(),
            _ => self.parse_expr_stmt(),
        };

        self.expect(Token::Semicolon)?;

        stmt
    }

    fn parse_print_stmt(&mut self) -> ParseResult<'src, Stmt<'src>> {
        self.lexer.next().unwrap();

        Ok(Stmt::Print(self.parse_expr()?))
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<'src, Stmt<'src>> {
        Ok(Stmt::Expr(self.parse_expr()?))
    }

    fn parse_expr(&mut self) -> ParseResult<'src> {
        self.parse_binop(0)
    }

    /// Parses binary operations using precedence climbing. This is conceptually the
    /// same thing as the book describes when parsing equality, comparisons, terms,
    /// and factors, but here we condense all of that into one step.
    fn parse_binop(&mut self, min_prec: u32) -> ParseResult<'src> {
        let mut expr = self.parse_unary()?;

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

            let rhs = self.parse_binop(next_min_prec)?;

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                op,
                right: Box::new(rhs),
            });
        }

        Ok(expr)
    }

    /// A trivial unary expression parser. It simply applies unary operators right-to-left
    fn parse_unary(&mut self) -> ParseResult<'src> {
        let mut ops = VecDeque::new();

        while let Some(op) = self.peek_unary_operator() {
            self.lexer.next();
            ops.push_front(op);
        }

        let mut expr = self.parse_atom()?;

        for op in ops.into_iter() {
            expr = Expr::Unary(Unary {
                op,
                right: Box::new(expr),
            });
        }

        Ok(expr)
    }

    /// Parses literals and groupings (parenthesized expressions)
    fn parse_atom(&mut self) -> ParseResult<'src> {
        let token = self
            .lexer
            .next()
            .ok_or(ParserError::new(None, ParserErrorKind::UnexpectedEof))?;
        Ok(match token.token {
            Token::NumLit(l) => Expr::Literal(Literal::NumLit(NumLit(l))),
            Token::StringLit(l) => Expr::Literal(Literal::StringLit(StringLit(l))),
            Token::Identifier(l) => Expr::Literal(Literal::Identifier(Identifier(l))),
            Token::Nil => Expr::Literal(Literal::Nil),
            Token::True => Expr::Literal(Literal::Bool(true)),
            Token::False => Expr::Literal(Literal::Bool(false)),
            Token::LeftParen => self.parse_paren_expr()?,
            _ => Err(Error::new(token, ParserErrorKind::UnexpectedToken))?,
        })
    }

    fn parse_paren_expr(&mut self) -> ParseResult<'src> {
        let expr = Expr::Grouping(Grouping {
            expr: Box::new(self.parse_expr()?),
        });
        self.expect(Token::RightParen)?;
        Ok(expr)
    }

    fn expect(&mut self, expected: Token<'src>) -> ParseResult<'src, Token<'src>> {
        let token = self
            .lexer
            .peek()
            .ok_or(Error::new(None, ParserErrorKind::UnexpectedEof))?;

        if token == expected {
            Ok(self.lexer.next().unwrap().token)
        } else {
            Err(Error::new(
                self.lexer.peek_spanned().unwrap(),
                ParserErrorKind::UnexpectedToken,
            ))
        }
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

pub type ParseResult<'src, T = Expr<'src>> = Result<T, ParserError<'src>>;

pub type ParserError<'src> = Error<'src, ParserErrorKind>;

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum ParserErrorKind {
    #[error("no rule expected token")]
    UnexpectedToken,
    #[error("unexpected end of file")]
    UnexpectedEof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expr_with_precedence() {
        let expr = Parser::new("1 >= 2 * 3 + 4").parse_expr().unwrap();
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

    #[test]
    fn parse_grouping() {
        let expr = Parser::new("1 + (2 + foo)").parse_expr().unwrap();
        dbg!(&expr);

        let expected = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal::NumLit(NumLit("1")))),
            right: Box::new(Expr::Grouping(Grouping {
                expr: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Literal(Literal::NumLit(NumLit("2")))),
                    right: Box::new(Expr::Literal(Literal::Identifier(Identifier("foo")))),
                    op: BinaryOp::Add,
                })),
            })),
            op: BinaryOp::Add,
        });

        assert_eq!(expr, expected);
    }
}
