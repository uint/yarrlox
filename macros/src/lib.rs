mod ast;

use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn define_ast(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ast::AstDef);
    let expanded_items = ast::define_ast(input);

    let expanded = quote! {
        #(#expanded_items)*
    };

    proc_macro::TokenStream::from(expanded)
}
