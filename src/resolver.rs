// In the book, a hash map was used for the resolution table. Variable expressions
// are uniquely identifiable in Java, so they can be addressed by their unique
// hashes. I didn't have that luxury in Rust. Instead, I made the parser assign
// a unique ID to every place that refers to a variable.
//
// Since those IDs are contiguous, we don't really need a hash map here.

use std::collections::{HashMap, VecDeque};

use crate::ast::*;

pub struct Resolver<'ast> {
    locals: Vec<usize>,
    scopes: VecDeque<HashMap<&'ast str, bool>>,
}

impl<'ast> Resolver<'ast> {
    pub fn new(len: usize) -> Self {
        let mut locals = Vec::with_capacity(len);

        unsafe {
            locals.set_len(len);
        }

        Self {
            locals,
            scopes: VecDeque::new(),
        }
    }

    fn resolve(&mut self, stmts: &'ast [Stmt]) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &'ast Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve(stmts);
                self.end_scope();
            }
            Stmt::Expr(expr) => self.resolve_expr(expr)?,
            Stmt::Function(fun) => self.resolve_fun_decl(fun),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(els) = else_branch {
                    self.resolve_stmt(els)?;
                }
            }
            Stmt::Print(expr) => self.resolve_expr(expr)?,
            Stmt::Var { name, initializer } => self.resolve_var_stmt(name, initializer.as_ref()),
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
            Stmt::Break => {}
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.resolve_expr(expr)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_fun_decl(&mut self, fun: &'ast Function) {
        self.declare(&fun.name);
        self.define(&fun.name);
        self.resolve_fun(fun);
    }

    fn resolve_fun(&mut self, fun: &'ast Function) {
        self.begin_scope();
        for param in &fun.params {
            self.declare(&param);
            self.define(&param);
        }
        self.resolve(&fun.body);
        self.end_scope();
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult {
        match expr {
            Expr::Assign(Assign { name, value }) => {
                self.resolve_expr(value)?;
                self.resolve_local(name);
            }
            Expr::Literal(Literal::Identifier(reference)) => self.resolve_var_expr(reference)?,
            Expr::Literal(_) => {}
            Expr::Binary(Binary { left, right, .. }) => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Unary(Unary { right, .. }) => self.resolve_expr(right)?,
            Expr::Grouping(Grouping { expr }) => self.resolve_expr(expr)?,
            Expr::Call(Call { callee, args, .. }) => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_var_stmt(&mut self, name: &'ast str, initializer: Option<&Expr>) {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve_expr(init);
        }
        self.define(name);
    }

    fn resolve_var_expr(&mut self, reference: &Reference) -> ResolverResult {
        if let Some(scope) = self.scopes.get(0) {
            if scope.get(reference.ident.as_str()) == Some(&false) {
                return Err(ResolverError::SelfInitialize);
            }
        }

        self.resolve_local(reference);

        Ok(())
    }

    fn resolve_assign_expr(&mut self, expr: &Assign) -> ResolverResult {
        self.resolve_expr(&expr.value)?;
        self.resolve_local(&expr.name);

        Ok(())
    }

    fn resolve_local(&mut self, reference: &Reference) {
        for (ix, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(reference.ident.as_str()) {
                self.locals[reference.id] = ix;
                return;
            }
        }
    }

    fn declare(&mut self, name: &'ast str) {
        if let Some(scope) = self.scopes.get_mut(0) {
            scope.insert(name, false);
        }
    }

    fn define(&mut self, name: &'ast str) {
        if let Some(scope) = self.scopes.get_mut(0) {
            scope.insert(name, true);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push_front(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop_front();
    }
}

type ResolverResult = Result<(), ResolverError>;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ResolverError {
    #[error("Can't read local variable in its own initializer.")]
    SelfInitialize,
}
