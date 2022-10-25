use std::iter::Peekable;

use logos::Logos;

use crate::token::{SpannedToken, Token};

pub struct Lexer<'src> {
    // The fact we use the `logos` lexer is an implementation detail of our `Lexer`.
    // We might want to change that in the future, so we encapsulate this detail.
    inner: Peekable<logos::SpannedIter<'src, Token<'src>>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            inner: Token::lexer(source).spanned().peekable(),
        }
    }

    pub fn peek(&mut self) -> Option<Token<'src>> {
        self.inner.peek().map(|(token, _span)| *token)
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = SpannedToken<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(token, span)| SpannedToken { token, span })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helpers::*;

    mod helpers {
        use super::super::*;

        pub(super) trait IntoTokens {
            fn into_tokens(self) -> Vec<Token<'static>>;
        }

        impl IntoTokens for &'static str {
            fn into_tokens(self) -> Vec<Token<'static>> {
                Lexer::new(self).into_iter().map(|t| t.token).collect()
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
                StringLit("asd"),
                Semicolon,
                Var,
                Identifier("bar"),
                Equal,
                StringLit("dsa"),
                Semicolon,
            ],
        );

        assert_lexer(
            r#"var foo = "👁💃🕺🈯️as  \n\\n \"d\""; var bar = "dsa";"#,
            [
                Var,
                Identifier("foo"),
                Equal,
                StringLit(r#"👁💃🕺🈯️as  \n\\n \"d\""#),
                Semicolon,
                Var,
                Identifier("bar"),
                Equal,
                StringLit("dsa"),
                Semicolon,
            ],
        );
    }

    #[test]
    fn integers() {
        use Token::*;

        assert_lexer("324", [NumLit("324")]);
    }

    #[test]
    fn decimals() {
        use Token::*;

        assert_lexer("324.5", [NumLit("324.5")]);
        assert_lexer("324.", [NumLit("324"), Dot]);
        assert_lexer(".5", [Dot, NumLit("5")]);
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
