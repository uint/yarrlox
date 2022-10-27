use std::ops::Range;

use logos::{Filter, Logos};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpannedToken<'src> {
    pub token: Token<'src>,
    pub span: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Logos)]
pub enum Token<'src> {
    // Single-character tokens.
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token("/")]
    Slash,
    #[token("*")]
    Star,

    // One or two character tokens.
    #[token("!")]
    Bang,
    #[token("!=")]
    BangEqual,
    #[token("=")]
    Equal,
    #[token("==")]
    EqualEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,

    // Literals.
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(&'src str),
    #[regex(r#""([^"\\]|\\.)*""#, callback = trim_string)]
    StringLit(&'src str),
    #[regex(r#"[0-9]+(\.[0-9]+)?"#)]
    NumLit(&'src str),

    // Keywords.
    #[token("and")]
    And,
    #[token("class")]
    Class,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("fun")]
    Fun,
    #[token("for")]
    For,
    #[token("if")]
    If,
    #[token("nil")]
    Nil,
    #[token("or")]
    Or,
    #[token("print")]
    Print,
    #[token("return")]
    Return,
    #[token("super")]
    Super,
    #[token("this")]
    This,
    #[token("true")]
    True,
    #[token("var")]
    Var,
    #[token("while")]
    While,

    // Errors, logic for skipping comments and whitespace
    #[error]
    #[regex(r#"//[^\n]*"#, logos::skip)] // ignore single line comments
    #[regex(r"[ \n\t\f]+", logos::skip)] // ignore whitespace
    InvalidToken,

    #[token("/*", skip_block_comment)]
    UnterminatedBlockComment,
}

fn skip_block_comment<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> Filter<()> {
    match lex.remainder().find("*/") {
        Some(ix) => {
            lex.bump(ix + 2);
            Filter::Skip
        }
        None => {
            lex.bump(lex.remainder().len());
            // emit UnterminatedBlockComment
            Filter::Emit(())
        }
    }
}

fn trim_string<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> &'src str {
    let s = lex.slice();
    &s[1..(s.len() - 1)]
}
