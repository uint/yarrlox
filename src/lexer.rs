use logos::Logos;

#[derive(Clone, Debug, PartialEq, Eq, Logos)]
pub enum Token<'source> {
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
    Identifier(&'source str),
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit(&'source str),
    #[regex("[0-9]+")]
    NumLit(&'source str),

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

    // Invalid tokens
    #[error]
    #[regex(r"[ \t\f]+", logos::skip)] // ignore whitespace
    Error,
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[track_caller]
    fn assert_lexer<'s>(src: &'s str, expected: impl IntoIterator<Item = Token<'s>>) {
        let mut lexer = Token::lexer(src);

        let expected: Vec<_> = expected.into_iter().collect();

        for expected in expected {
            let token = lexer.next().unwrap();
            if token != expected {
                panic!(
                    "Unexpected token. Expected: {:?}, got: {}",
                    expected,
                    lexer.slice()
                );
            }
        }
    }

    #[test]
    fn tokenize_strings() {
        use Token::*;

        assert_lexer(
            r#"var foo = "asd"; var bar = "dsa";"#,
            [
                Var,
                Identifier("foo"),
                Equal,
                StringLit("\"asd\""),
                Semicolon,
                Var,
                Identifier("bar"),
                Equal,
                StringLit("\"dsa\""),
                Semicolon,
            ],
        );

        assert_lexer(
            r#"var foo = "as \n\\n \"d\""; var bar = "dsa";"#,
            [
                Var,
                Identifier("foo"),
                Equal,
                StringLit(r#""as \n\\n \"d\"""#),
                Semicolon,
                Var,
                Identifier("bar"),
                Equal,
                StringLit("\"dsa\""),
                Semicolon,
            ],
        );
    }
}
