use std::collections::VecDeque;

use crate::ast::*;
use crate::errors::Error;
use crate::lexer::Lexer;
use crate::token::{SpannedToken, Token};

#[derive(Default)]
pub struct Parser {
    loop_depth: u32,
    next_var_expr_id: usize,
}

impl<'src> Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn var_count(&self) -> usize {
        self.next_var_expr_id
    }

    pub fn parse(&mut self, src: &'src str) -> Result<Vec<Stmt>, Vec<ParserError<'src>>> {
        let mut lexer = Lexer::new(src);

        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        while !Self::is_at_end(&mut lexer) {
            match self.parse_decl(&mut lexer) {
                Ok(decl) => stmts.push(decl),
                Err(err) => {
                    self.synchronize(&mut lexer);
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

    /// Recover from a syntax error by entering panic mode and discarding tokens
    /// until a (likely) statement boundary is found.
    fn synchronize(&mut self, lexer: &mut Lexer<'src>) {
        if let Some(mut previous) = lexer.next() {
            while let Some(token) = lexer.peek() {
                use Token::*;

                if previous.token == Semicolon {
                    return;
                }

                match token {
                    Class | Fun | Var | For | If | While | Print | Return => return,
                    _ => previous = lexer.next().unwrap(),
                }
            }
        }
    }

    fn is_at_end(lexer: &mut Lexer<'src>) -> bool {
        lexer.peek().is_none()
    }

    fn parse_decl(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        let res = match lexer.peek().unwrap() {
            Token::Fun => self.parse_fun_decl(lexer)?,
            Token::Var => self.parse_var_decl(lexer)?,
            _ => self.parse_stmt(lexer)?,
        };

        Ok(res)
    }

    fn parse_fun_decl(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();

        let name = self.parse_ident(lexer)?;

        self.expect(lexer, Token::LeftParen)?;

        let mut params = vec![];
        if lexer.peek() != Some(Token::RightParen) {
            while {
                if params.len() >= 255 {
                    return Err(Error::new(
                        lexer.peek_spanned(),
                        ParserErrorKind::TooManyArgs,
                    ));
                }
                params.push(self.parse_ident(lexer)?);
                self.expect(lexer, Token::Comma).is_ok()
            } {}
        }

        self.expect(lexer, Token::RightParen)?;
        self.check(lexer, Token::LeftBrace)?;

        let body = self.parse_block(lexer)?;

        Ok(Stmt::Function(Function { name, params, body }))
    }

    fn parse_ident(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, String> {
        match lexer.peek() {
            Some(Token::Identifier(ident)) => {
                lexer.next().unwrap();
                Ok(ident.to_string())
            }
            Some(_) => Err(Error::new(
                lexer.peek_spanned(),
                ParserErrorKind::UnexpectedToken,
            )),
            None => Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        }
    }

    fn parse_var_decl(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();

        let res = match lexer.next() {
            Some(SpannedToken {
                token: Token::Identifier(ident),
                ..
            }) => Ok(Stmt::Var {
                name: ident.to_string(),
                initializer: if self.expect(lexer, Token::Equal).is_ok() {
                    Some(self.parse_expr(lexer)?)
                } else {
                    None
                },
            }),
            token => Err(Error::new(token, ParserErrorKind::UnexpectedToken)),
        };

        self.expect(lexer, Token::Semicolon)?;

        res
    }

    fn parse_stmt(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        match lexer.peek().unwrap() {
            Token::For => self.parse_for_loop(lexer),
            Token::If => self.parse_if(lexer),
            Token::Print => self.parse_print_stmt(lexer),
            Token::Return => self.parse_return_stmt(lexer),
            Token::While => self.parse_while_loop(lexer),
            Token::LeftBrace => Ok(Stmt::Block(self.parse_block(lexer)?)),
            Token::Break => self.parse_break(lexer),
            _ => self.parse_expr_stmt(lexer),
        }
    }

    fn parse_for_loop(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();

        self.expect(lexer, Token::LeftParen)?;

        let initializer = match lexer.peek() {
            Some(Token::Semicolon) => {
                lexer.next().unwrap();
                None
            }
            Some(Token::Var) => Some(self.parse_var_decl(lexer)?),
            Some(_) => Some(self.parse_expr_stmt(lexer)?),
            None => return Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        };

        let condition = match lexer.peek() {
            Some(Token::Semicolon) => Expr::Literal(Literal::Bool(true)),
            Some(_) => self.parse_expr(lexer)?,
            None => return Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        };

        self.expect(lexer, Token::Semicolon)?;

        let increment = match lexer.peek() {
            Some(Token::RightParen) => None,
            Some(_) => Some(self.parse_expr(lexer)?),
            None => return Err(Error::new(None, ParserErrorKind::UnexpectedEof)),
        };

        self.expect(lexer, Token::RightParen)?;

        self.loop_depth += 1;
        let body = self.parse_stmt(lexer);
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

    fn parse_if(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();

        self.expect(lexer, Token::LeftParen)?;
        let condition = self.parse_expr(lexer)?;
        self.expect(lexer, Token::RightParen)?;

        let then_branch = Box::new(self.parse_stmt(lexer)?);
        let else_branch = if self.expect(lexer, Token::Else).is_ok() {
            Some(Box::new(self.parse_stmt(lexer)?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_loop(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();

        self.expect(lexer, Token::LeftParen)?;
        let condition = self.parse_expr(lexer)?;
        self.expect(lexer, Token::RightParen)?;

        self.loop_depth += 1;
        let body = self.parse_stmt(lexer);
        self.loop_depth -= 1;
        let body = Box::new(body?);

        Ok(Stmt::While { condition, body })
    }

    fn parse_print_stmt(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();
        let res = self.parse_expr(lexer)?;

        self.expect(lexer, Token::Semicolon)?;

        Ok(Stmt::Print(res))
    }

    fn parse_return_stmt(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        lexer.next().unwrap();
        let val = if lexer.peek() == Some(Token::Semicolon) {
            None
        } else {
            Some(self.parse_expr(lexer)?)
        };

        self.expect(lexer, Token::Semicolon)?;

        Ok(Stmt::Return(val))
    }

    fn parse_block(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Vec<Stmt>> {
        lexer.next().unwrap();

        let mut stmts = vec![];
        while !matches!(lexer.peek(), Some(Token::RightBrace) | None) {
            stmts.push(self.parse_decl(lexer)?);
        }

        lexer
            .next()
            .ok_or_else(|| Error::new(None, ParserErrorKind::UnexpectedEof))?;

        Ok(stmts)
    }

    fn parse_break(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        let break_token = lexer.next().unwrap();

        self.expect(lexer, Token::Semicolon)?;

        match self.loop_depth {
            0 => Err(Error::new(
                Some(break_token),
                ParserErrorKind::BreakOutsideLoop,
            )),
            _ => Ok(Stmt::Break),
        }
    }

    fn parse_expr_stmt(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src, Stmt> {
        let res = self.parse_expr(lexer)?;
        self.expect(lexer, Token::Semicolon)?;
        Ok(Stmt::Expr(res))
    }

    fn parse_expr(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src> {
        self.parse_assignment(lexer)
    }

    fn parse_assignment(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src> {
        let expr = self.parse_binop(lexer, 0)?;

        if let Ok(equals) = self.expect(lexer, Token::Equal) {
            let value = self.parse_assignment(lexer)?;

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
    fn parse_binop(&mut self, lexer: &mut Lexer<'src>, min_prec: u32) -> ParseResult<'src> {
        let mut expr = self.parse_unary(lexer)?;

        while let Some(op) = self.peek_binary_operator(lexer) {
            let (prec, assoc) = op.prec_assoc();

            if prec < min_prec {
                break;
            }

            // consume the operator token
            lexer.next();

            let next_min_prec = match assoc {
                Assoc::Left => prec + 1,
                Assoc::Right => prec,
            };

            let rhs = self.parse_binop(lexer, next_min_prec)?;

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                op,
                right: Box::new(rhs),
            });
        }

        Ok(expr)
    }

    /// A trivial unary expression parser. It simply applies unary operators right-to-left
    fn parse_unary(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src> {
        let mut ops = VecDeque::new();

        while let Some(op) = self.peek_unary_operator(lexer) {
            lexer.next();
            ops.push_front(op);
        }

        let mut expr = self.parse_call(lexer)?;

        for op in ops.into_iter() {
            expr = Expr::Unary(Unary {
                op,
                right: Box::new(expr),
            });
        }

        Ok(expr)
    }

    fn parse_call(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src> {
        let mut expr = self.parse_atom(lexer)?;

        loop {
            if self.expect(lexer, Token::LeftParen).is_ok() {
                expr = self.finish_call(lexer, expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, lexer: &mut Lexer<'src>, callee: Expr) -> ParseResult<'src> {
        let mut args = vec![];
        if lexer
            .peek()
            .ok_or_else(|| Error::new(None, ParserErrorKind::UnexpectedEof))?
            != Token::RightParen
        {
            while {
                if args.len() >= 255 {
                    return Err(Error::new(
                        lexer.peek_spanned().unwrap(),
                        ParserErrorKind::TooManyArgs,
                    ));
                }
                args.push(self.parse_expr(lexer)?);
                self.expect(lexer, Token::Comma).is_ok()
            } {}
        }

        let paren = self.expect(lexer, Token::RightParen)?;

        Ok(Expr::Call(Call {
            args,
            callee: Box::new(callee),
            paren: paren.span,
        }))
    }

    /// Parses literals and groupings (parenthesized expressions)
    fn parse_atom(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src> {
        let token = lexer
            .next()
            .ok_or_else(|| ParserError::new(None, ParserErrorKind::UnexpectedEof))?;
        Ok(match token.token {
            Token::NumLit(l) => Expr::Literal(Literal::NumLit(NumLit(l.to_string()))),
            Token::StringLit(l) => Expr::Literal(Literal::StringLit(StringLit(l.to_string()))),
            Token::Identifier(l) => self.make_var_expr(l),
            Token::Nil => Expr::Literal(Literal::Nil),
            Token::True => Expr::Literal(Literal::Bool(true)),
            Token::False => Expr::Literal(Literal::Bool(false)),
            Token::LeftParen => self.parse_paren_expr(lexer)?,
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

    fn parse_paren_expr(&mut self, lexer: &mut Lexer<'src>) -> ParseResult<'src> {
        let expr = Expr::Grouping(Grouping {
            expr: Box::new(self.parse_expr(lexer)?),
        });
        self.expect(lexer, Token::RightParen)?;
        Ok(expr)
    }

    fn expect(
        &mut self,
        lexer: &mut Lexer<'src>,
        expected: Token<'src>,
    ) -> ParseResult<'src, SpannedToken<'src>> {
        let token = lexer
            .peek()
            .ok_or_else(|| Error::new(None, ParserErrorKind::UnexpectedEof))?;

        if token == expected {
            Ok(lexer.next().unwrap())
        } else {
            Err(Error::new(
                lexer.peek_spanned().unwrap(),
                ParserErrorKind::UnexpectedToken,
            ))
        }
    }

    fn check(
        &mut self,
        lexer: &mut Lexer<'src>,
        expected: Token<'src>,
    ) -> ParseResult<'src, SpannedToken<'src>> {
        let token = lexer
            .peek()
            .ok_or_else(|| Error::new(None, ParserErrorKind::UnexpectedEof))?;

        if token == expected {
            Ok(lexer.peek_spanned().unwrap())
        } else {
            Err(Error::new(
                lexer.peek_spanned().unwrap(),
                ParserErrorKind::UnexpectedToken,
            ))
        }
    }

    fn peek_binary_operator(&mut self, lexer: &mut Lexer<'src>) -> Option<BinaryOp> {
        match lexer.peek() {
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

    fn peek_unary_operator(&mut self, lexer: &mut Lexer<'src>) -> Option<UnaryOp> {
        match lexer.peek() {
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
        let mut lexer = Lexer::new("1 >= 2 * 3 + 4");
        let expr = Parser::new().parse_expr(&mut lexer).unwrap();
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
        let mut lexer = Lexer::new("1 + (2 + foo)");
        let expr = Parser::new().parse_expr(&mut lexer).unwrap();
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
