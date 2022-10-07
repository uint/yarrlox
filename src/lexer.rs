use logos::{Filter, Lexer, Logos};

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

    // Errors, logic for skipping comments and whitespace
    #[error]
    #[regex(r#"//[^\n]*"#, logos::skip)] // ignore single line comments
    #[regex(r"[ \n\t\f]+", logos::skip)] // ignore whitespace
    InvalidToken,

    #[token("/*", skip_block_comment)]
    UnterminatedBlockComment,
}

fn skip_block_comment<'source>(lex: &mut Lexer<'source, Token<'source>>) -> Filter<()> {
    if let Some(ix) = lex.remainder().find("*/") {
        lex.bump(ix + 2);
        Filter::Skip
    } else {
        // emit UnterminatedBlockComment
        Filter::Emit(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helpers::*;

    mod helpers {
        use super::super::*;

        pub trait IntoTokens {
            fn into_tokens(self) -> Vec<Token<'static>>;
        }

        impl IntoTokens for &'static str {
            fn into_tokens(self) -> Vec<Token<'static>> {
                Token::lexer(self).into_iter().collect()
            }
        }

        impl<const N: usize> IntoTokens for [Token<'static>; N] {
            fn into_tokens(self) -> Vec<Token<'static>> {
                self.to_vec()
            }
        }
    }

    #[track_caller]
    fn assert_lexer(left: impl IntoTokens, right: impl IntoTokens) {
        let left_vec = left.into_tokens();
        let right_vec = right.into_tokens();

        for (left, right) in left_vec.clone().into_iter().zip(right_vec.clone()) {
            if left != right {
                panic!(
                    "tokens didn't match: left: {:?}, right: {:?}\nleft : {:?}\nright: {:?}",
                    left, right, left_vec, right_vec
                );
            }
        }
    }

    #[test]
    fn string_literals() {
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
            r#"var foo = "👁💃🕺🈯️as  \n\\n \"d\""; var bar = "dsa";"#,
            [
                Var,
                Identifier("foo"),
                Equal,
                StringLit(r#""👁💃🕺🈯️as  \n\\n \"d\"""#),
                Semicolon,
                Var,
                Identifier("bar"),
                Equal,
                StringLit("\"dsa\""),
                Semicolon,
            ],
        );
    }

    #[test]
    fn whitespace() {
        use Token::*;

        assert_lexer("var var", [Var, Var]);
        assert_lexer("var var", "var    var");
        assert_lexer("var var", "\n\t\t\nvar  \n\n\t  var ");
    }

    #[test]
    fn ignore_line_comments() {
        use Token::*;

        assert_lexer("var var // () \n var", [Var, Var, Var]);
    }

    #[test]
    fn ignore_block_comments() {
        use Token::*;

        assert_lexer("var var /* var a * / = 👾🈯️ 5; */ ;", [Var, Var, Semicolon]);
    }

    #[test]
    fn unterminated_block_comments() {
        use Token::*;

        assert_lexer(
            "var var /* var a = 5;",
            [Var, Var, UnterminatedBlockComment],
        );
    }
}
