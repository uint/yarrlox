use syn::{braced, parse_quote, Generics, Ident, Item, ItemStruct, Token, Variant};

pub fn define_ast(input: AstDef) -> Vec<Item> {
    match input {
        AstDef::Struct(struct_def) => {
            vec![parse_quote! {pub #struct_def}]
        }
        AstDef::Enum {
            ident,
            generics,
            content,
        } => {
            let variants = content.iter().map(|v| -> Variant {
                let ident = v.ident();
                let generics = v.generics();
                parse_quote! {#ident(#ident #generics)}
            });
            let mut defs = vec![Item::Enum(parse_quote! {pub enum #ident #generics {
                #(#variants),*
            }})];
            defs.extend(content.into_iter().map(define_ast).flatten());
            defs
        }
    }
}

pub enum AstDef {
    Struct(ItemStruct),
    Enum {
        ident: Ident,
        generics: Generics,
        content: Vec<AstDef>,
    },
}

impl AstDef {
    fn ident(&self) -> &Ident {
        match self {
            AstDef::Struct(s) => &s.ident,
            AstDef::Enum { ident, .. } => ident,
        }
    }

    fn generics(&self) -> &Generics {
        match self {
            AstDef::Struct(s) => &s.generics,
            AstDef::Enum { generics, .. } => generics,
        }
    }
}

impl syn::parse::Parse for AstDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![struct]) {
            Ok(Self::Struct(input.parse()?))
        } else if input.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            let ident: Ident = input.parse()?;
            let generics: Generics = input.parse()?;
            let content_raw;
            braced!(content_raw in input);
            let mut content = vec![];
            while !content_raw.is_empty() {
                content.push(content_raw.parse()?);
            }

            Ok(Self::Enum {
                ident,
                generics,
                content,
            })
        } else {
            Err(syn::Error::new(input.span(), "expected `enum` or `struct`"))
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn basic_struct() {
        let result = define_ast(parse_quote! {
            struct Expr {
                x: u32,
                y: String,
            }
        });

        assert_eq!(
            result,
            [parse_quote! {
                pub struct Expr {
                    x: u32,
                    y: String,
                }
            }]
        );
    }

    #[test]
    fn empty_enum() {
        let result = define_ast(parse_quote! {
            enum Expr {}
        });

        assert_eq!(
            result,
            [parse_quote! {
                pub enum Expr {}
            }]
        );
    }

    #[test]
    fn basic_enum() {
        let result = define_ast(parse_quote! {
            enum Expr {
                struct Foo;
                struct Bar(u32);
                struct Baz {x: String}
            }
        });

        assert_eq!(
            result,
            [
                parse_quote! {
                    pub enum Expr {
                        Foo(Foo),
                        Bar(Bar),
                        Baz(Baz)
                    }
                },
                parse_quote! {
                    pub struct Foo;
                },
                parse_quote! {
                    pub struct Bar(u32);
                },
                parse_quote! {
                    pub struct Baz {x: String}
                }
            ]
        );
    }

    #[test]
    fn enum_with_lifetimes() {
        let result = define_ast(parse_quote! {
            enum Expr<'src> {
                struct Foo;
                struct Bar<'src>(&'src str);
                struct Baz<'src> {x: &'src str}
            }
        });

        assert_eq!(
            result,
            [
                parse_quote! {
                    pub enum Expr<'src> {
                        Foo(Foo),
                        Bar(Bar<'src>),
                        Baz(Baz<'src>)
                    }
                },
                parse_quote! {
                    pub struct Foo;
                },
                parse_quote! {
                    pub struct Bar<'src>(&'src str);
                },
                parse_quote! {
                    pub struct Baz<'src> {x: &'src str}
                }
            ]
        );
    }
}
