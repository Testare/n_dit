mod conv;
mod error;
mod parse;
mod tokenize;
mod validate;

use crate::error::CharmiError;

use charmi_core::CharmiDef;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::parse_macro_input;

struct CharmiStatic(CharmiDef);

enum CharmiRepr {
    Dyn(CharmiDynamic),
    Stat(CharmiStatic),
}

struct CharmiDynamic(Vec<CharmiDynamicExpr>);

struct CharmiDynamicExpr {
    key: CharmiDynamicKey,
    span: Span,
    value: TokenStream2,
    // ExprAssign?
    // ExprField or
    // key: syn::
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum CharmiDynamicKey {
    Text,
    Fg,
    Bg,
    Split,
    Attr,
    Values,
    VGap,
    VSplit,
    VColors,
    VAttr,
    VColorKey(char),
    VAttrKey(char),
}

#[proc_macro]
pub fn charmi_toml(tokens: TokenStream) -> TokenStream {
    let repr = parse_macro_input!(tokens as CharmiRepr);
    quote!(#repr).into()
}
