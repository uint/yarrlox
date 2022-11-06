use std::collections::VecDeque;

use crate::ast::*;
use crate::errors::Error;
use crate::lexer::Lexer;
use crate::token::{SpannedToken, Token};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    loop_depth: u32,
    next_var_expr_id: u32,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            lexer: Lexer::new(src),
            loop_depth: 0,
            next_var_expr_id: 0,
        }
    }

    /// Recover from a syntax error by entering panic mode and discarding tokens
    /// until a (likely) statement boundary is found.
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError<'src>>> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.parse_decl() {
                Ok(decl) => stmts.push(decl),
                Err(err) => {
                    self.synchronize();
                    errors.push(err)
                }
            }
        }

        if errors.is_empty() {
            Ok(stmts)
        } else {
            Err(errors)
        }
    }

    fn parse_decl(&mut self) -> ParseResult<'src, Stmt> {
        let res = match self.lexer.peek().unwrap() {
            Token::Fun => self.parse_fun_decl()?,
            Token::Var => self.parse_var_decl()?,
            _ => self.parse_stmt()?,
        };

        Ok(res)
    }

    fn parse_fun_decl(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();

        let name = self.parse_ident()?;

        self.expect(Token::LeftParen)?;

        let mut params = vec![];
        if self.lexer.peek() != Some(Token::RightParen) {
            while {
                if params.len() >= 255 {
                    return Err(Error::new(
                        self.lexer.peek_spanned(),
                        ParserErrorKind::TooManyArgs,
                    ));
                }
                params.push(self.parse_ident()?);
                self.expect(Token::Comma).is_ok()
            } {}
        }

        self.expect(Token::RightParen)?;
        self.check(Token::LeftBrace)?;

        let body = self.parse_block()?;

        Ok(Stmt::Function(Function { name, params, body }))
    }

    fn parse_ident(&mut self) -> ParseResult<'src, String> {
        match self.lexer.peek() {
            Some(Token::Identifier(ident)) => {
                self.lexer.next().unwrap();
                Ok(ident.to_string())
            }
            Some(_) => Err(Error::new(
                self.lexer.peek_spanned(),
                ParserErrorKind::UnexpectedToken,
            )),
            None => Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        }
    }

    fn parse_ref(&mut self) -> ParseResult<'src, Reference> {
        match self.lexer.peek() {
            Some(Token::Identifier(ident)) => {
                self.lexer.next().unwrap();
                let reference = Reference {
                    id: self.next_var_expr_id,
                    ident: ident.to_string(),
                };
                self.next_var_expr_id += 1;

                Ok(reference)
            }
            Some(_) => Err(Error::new(
                self.lexer.peek_spanned(),
                ParserErrorKind::UnexpectedToken,
            )),
            None => Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        }
    }

    fn parse_var_decl(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();

        let res = match self.lexer.next() {
            Some(SpannedToken {
                token: Token::Identifier(ident),
                ..
            }) => Ok(Stmt::Var {
                name: ident.to_string(),
                initializer: if self.expect(Token::Equal).is_ok() {
                    Some(self.parse_expr()?)
                } else {
                    None
                },
            }),
            token => Err(Error::new(token, ParserErrorKind::UnexpectedToken)),
        };

        self.expect(Token::Semicolon)?;

        res
    }

    fn parse_stmt(&mut self) -> ParseResult<'src, Stmt> {
        match self.lexer.peek().unwrap() {
            Token::For => self.parse_for_loop(),
            Token::If => self.parse_if(),
            Token::Print => self.parse_print_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::While => self.parse_while_loop(),
            Token::LeftBrace => Ok(Stmt::Block(self.parse_block()?)),
            Token::Break => self.parse_break(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_for_loop(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();

        self.expect(Token::LeftParen)?;

        let initializer = match self.lexer.peek() {
            Some(Token::Semicolon) => {
                self.lexer.next().unwrap();
                None
            }
            Some(Token::Var) => Some(self.parse_var_decl()?),
            Some(_) => Some(self.parse_expr_stmt()?),
            None => return Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        };

        let condition = match self.lexer.peek() {
            Some(Token::Semicolon) => Expr::Literal(Literal::Bool(true)),
            Some(_) => self.parse_expr()?,
            None => return Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        };

        self.expect(Token::Semicolon)?;

        let increment = match self.lexer.peek() {
            Some(Token::RightParen) => None,
            Some(_) => Some(self.parse_expr()?),
            None => return Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        };

        self.expect(Token::RightParen)?;

        self.loop_depth += 1;
        let body = self.parse_stmt();
        self.loop_depth -= 1;
        let mut body = body?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(inc)]);
        }

        body = Stmt::While {
            condition,
            body: body.into(),
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }

        Ok(body)
    }

    fn parse_if(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();

        self.expect(Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::RightParen)?;

        let then_branch = Box::new(self.parse_stmt()?);
        let else_branch = if self.expect(Token::Else).is_ok() {
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_loop(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();

        self.expect(Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::RightParen)?;

        self.loop_depth += 1;
        let body = self.parse_stmt();
        self.loop_depth -= 1;
        let body = Box::new(body?);

        Ok(Stmt::While { condition, body })
    }

    fn parse_print_stmt(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();
        let res = self.parse_expr()?;

        self.expect(Token::Semicolon)?;

        Ok(Stmt::Print(res))
    }

    fn parse_return_stmt(&mut self) -> ParseResult<'src, Stmt> {
        self.lexer.next().unwrap();
        let val = if self.lexer.peek() == Some(Token::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        self.expect(Token::Semicolon)?;

        Ok(Stmt::Return(val))
    }

    fn parse_block(&mut self) -> ParseResult<'src, Vec<Stmt>> {
        self.lexer.next().unwrap();

        let mut stmts = vec![];
        while !matches!(self.lexer.peek(), Some(Token::RightBrace) | None) {
            stmts.push(self.parse_decl()?);
        }

        self.lexer
            .next()
            .ok_or_else(|| Error::new(None, ParserErrorKind::UnexpectedEof))?;

        Ok(stmts)
    }

    fn parse_break(&mut self) -> ParseResult<'src, Stmt> {
        let break_token = self.lexer.next().unwrap();

        self.expect(Token::Semicolon)?;

        match self.loop_depth {
            0 => Err(Error::new(
                Some(break_token),
                ParserErrorKind::BreakOutsideLoop,
            )),
            _ => Ok(Stmt::Break),
        }
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<'src, Stmt> {
        let res = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Expr(res))
    }

    fn parse_expr(&mut self) -> ParseResult<'src> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseResult<'src> {
        let expr = self.parse_binop(0)?;

        if let Ok(equals) = self.expect(Token::Equal) {
            let value = self.parse_assignment()?;

            if let Expr::Literal(Literal::Identifier(name)) = expr {
                return Ok(Expr::Assign(Assign {
                    name,
                    value: Box::new(value),
                }));
            } else {
                return Err(Error::new(equals, ParserErrorKind::InvalidLvalue));
            }
        }

        Ok(expr)
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

        let mut expr = self.parse_call()?;

        for op in ops.into_iter() {
            expr = Expr::Unary(Unary {
                op,
                right: Box::new(expr),
            });
        }

        Ok(expr)
    }

    fn parse_call(&mut self) -> ParseResult<'src> {
        let mut expr = self.parse_atom()?;

        loop {
            if self.expect(Token::LeftParen).is_ok() {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<'src> {
        let mut args = vec![];
        if self
            .lexer
            .peek()
            .ok_or_else(|| Error::new(None, ParserErrorKind::UnexpectedEof))?
            != Token::RightParen
        {
            while {
                if args.len() >= 255 {
                    return Err(Error::new(
                        self.lexer.peek_spanned().unwrap(),
                        ParserErrorKind::TooManyArgs,
                    ));
                }
                args.push(self.parse_expr()?);
                self.expect(Token::Comma).is_ok()
            } {}
        }

        let paren = self.expect(Token::RightParen)?;

        Ok(Expr::Call(Call {
            args,
            callee: Box::new(callee),
            paren: paren.span,
        }))
    }

    /// Parses literals and groupings (parenthesized expressions)
    fn parse_atom(&mut self) -> ParseResult<'src> {
        let token = self
            .lexer
            .next()
            .ok_or_else(|| ParserError::new(None, ParserErrorKind::UnexpectedEof))?;
        Ok(match token.token {
            Token::NumLit(l) => Expr::Literal(Literal::NumLit(NumLit(l.to_string()))),
            Token::StringLit(l) => Expr::Literal(Literal::StringLit(StringLit(l.to_string()))),
            Token::Identifier(l) => self.make_var_expr(l),
            Token::Nil => Expr::Literal(Literal::Nil),
            Token::True => Expr::Literal(Literal::Bool(true)),
            Token::False => Expr::Literal(Literal::Bool(false)),
            Token::LeftParen => self.parse_paren_expr()?,
            _ => Err(Error::new(token, ParserErrorKind::UnexpectedToken))?,
        })
    }

    fn make_var_expr(&mut self, ident: &str) -> Expr {
        let expr = Expr::Literal(Literal::Identifier(Reference {
            id: self.next_var_expr_id,
            ident: ident.to_string(),
        }));

        self.next_var_expr_id += 1;

        expr
    }

    fn parse_paren_expr(&mut self) -> ParseResult<'src> {
        let expr = Expr::Grouping(Grouping {
            expr: Box::new(self.parse_expr()?),
        });
        self.expect(Token::RightParen)?;
        Ok(expr)
    }

    fn expect(&mut self, expected: Token<'src>) -> ParseResult<'src, SpannedToken<'src>> {
        let token = self
            .lexer
            .peek()
            .ok_or(Error::new(None, ParserErrorKind::UnexpectedEof))?;

        if token == expected {
            Ok(self.lexer.next().unwrap())
        } else {
            Err(Error::new(
                self.lexer.peek_spanned().unwrap(),
                ParserErrorKind::UnexpectedToken,
            ))
        }
    }

    fn check(&mut self, expected: Token<'src>) -> ParseResult<'src, SpannedToken<'src>> {
        let token = self
            .lexer
            .peek()
            .ok_or(Error::new(None, ParserErrorKind::UnexpectedEof))?;

        if token == expected {
            Ok(self.lexer.peek_spanned().unwrap())
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
            Some(Token::Or) => Some(BinaryOp::LogicOr),
            Some(Token::And) => Some(BinaryOp::LogicAnd),
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
            BinaryOp::LogicOr => (0, Assoc::Left),
            BinaryOp::LogicAnd => (1, Assoc::Left),
            BinaryOp::Eq => (2, Assoc::Left),
            BinaryOp::NotEq => (2, Assoc::Left),
            BinaryOp::Gt => (3, Assoc::Left),
            BinaryOp::Lt => (3, Assoc::Left),
            BinaryOp::Gte => (3, Assoc::Left),
            BinaryOp::Lte => (3, Assoc::Left),
            BinaryOp::Add => (4, Assoc::Left),
            BinaryOp::Sub => (4, Assoc::Left),
            BinaryOp::Mul => (5, Assoc::Left),
            BinaryOp::Div => (5, Assoc::Left),
        }
    }
}

pub type ParseResult<'src, T = Expr> = Result<T, ParserError<'src>>;

pub type ParserError<'src> = Error<'src, ParserErrorKind>;

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum ParserErrorKind {
    #[error("no rule expected token")]
    UnexpectedToken,
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("invalid l-value")]
    InvalidLvalue,
    #[error("trying to use `break` outside any loop")]
    BreakOutsideLoop,
    #[error("a function call can only accept up to 255 args")]
    TooManyArgs,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expr_with_precedence() {
        let expr = Parser::new("1 >= 2 * 3 + 4").parse_expr().unwrap();
        dbg!(&expr);

        let expected = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal::NumLit(NumLit("1".to_string())))),
            right: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Literal(Literal::NumLit(NumLit("2".to_string())))),
                    right: Box::new(Expr::Literal(Literal::NumLit(NumLit("3".to_string())))),
                    op: BinaryOp::Mul,
                })),
                right: Box::new(Expr::Literal(Literal::NumLit(NumLit("4".to_string())))),
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
            left: Box::new(Expr::Literal(Literal::NumLit(NumLit("1".to_string())))),
            right: Box::new(Expr::Grouping(Grouping {
                expr: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Literal(Literal::NumLit(NumLit("2".to_string())))),
                    right: Box::new(Expr::Literal(Literal::Identifier(Reference {
                        ident: "foo".to_string(),
                        id: 0,
                    }))),
                    op: BinaryOp::Add,
                })),
            })),
            op: BinaryOp::Add,
        });

        assert_eq!(expr, expected);
    }
}
