use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;

#[proc_macro]
pub fn charmi_toml(input: TokenStream) -> TokenStream {
    let ast: LitStr = syn::parse(input).unwrap();
    let ast_str = ast.value();
    let char_things: proc_macro2::TokenStream = ast_str
        .chars()
        .map(|ch| {
            quote!(
                Some(charmi::CharmiCell {
                    character: Some(#ch) ,
                    fg: None,
                    bg: None,
                }),
            )
        })
        .collect();
    TokenStream::from(quote! {
        charmi::CharmiStr::from_slice(
        &[#char_things]
        )
    })
}

#[proc_macro]
pub fn charmi_str(input: TokenStream) -> TokenStream {
    let ast: LitStr = syn::parse(input).unwrap();
    let ast_str = ast.value();
    let char_things: proc_macro2::TokenStream = ast_str
        .chars()
        .map(|ch| {
            quote!(
                Some(charmi::CharmiCell {
                    character: Some(#ch) ,
                    fg: None,
                    bg: None,
                }),
            )
        })
        .collect();
    TokenStream::from(quote! {
        charmi::CharmiStr::from_slice(
        &[#char_things]
        )
    })
}
