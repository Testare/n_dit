use crate::{CharmiDynamicExpr, CharmiDynamicKey};

use charmi_core::{CharmiDef, ColorDef, definition::Values};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

pub fn def_to_dyn_expr(span: Span, charmi_def: CharmiDef) -> Vec<CharmiDynamicExpr> {
    let CharmiDef {
        fg,
        bg,
        text,
        attr,
        values,
        split,
    } = charmi_def;

    let mut results = Vec::new();

    if bg.is_some() {
        results.push(CharmiDynamicExpr {
            span,
            key: CharmiDynamicKey::Bg,
            value: quote_spanned!(span=>Some(String::from(#bg))),
        });
    }
    if fg.is_some() {
        results.push(CharmiDynamicExpr {
            span,
            key: CharmiDynamicKey::Fg,
            value: quote_spanned!(span=>Some(String::from(#fg))),
        });
    }
    if text.is_some() {
        results.push(CharmiDynamicExpr {
            span,
            key: CharmiDynamicKey::Text,
            value: quote_spanned!(span=>Some(String::from(#text))),
        });
    }
    if split.is_some() {
        results.push(CharmiDynamicExpr {
            span,
            key: CharmiDynamicKey::Split,
            value: quote_spanned!(span=>Some(String::from(#text))),
        });
    }
    if attr.is_some() {
        results.push(CharmiDynamicExpr {
            span,
            key: CharmiDynamicKey::Attr,
            value: quote_spanned!(span=>Some(String::from(#attr))),
        });
    }
    if let Some(Values {
        gap,
        colors,
        attr,
        split,
    }) = values
    {
        if gap.is_some() {
            results.push(CharmiDynamicExpr {
                span,
                key: CharmiDynamicKey::VGap,
                value: quote_spanned!(span=>#gap),
            });
        }
        if split.is_some() {
            results.push(CharmiDynamicExpr {
                span,
                key: CharmiDynamicKey::VSplit,
                value: quote!(#split),
            });
        }
        if let Some(attr) = attr {
            results.extend(attr.into_iter().map(|(k, v)| CharmiDynamicExpr {
                span,
                key: CharmiDynamicKey::VAttrKey(k),
                value: quote!(#v),
            }));
        }
        if let Some(colors) = colors {
            results.extend(colors.into_iter().map(|(k, v)| CharmiDynamicExpr {
                span,
                key: CharmiDynamicKey::VColorKey(k),
                value: color_def_to_tokens(span, v),
            }));
        }
    }

    results
}

fn color_def_to_tokens(span: Span, color: ColorDef) -> TokenStream {
    match color {
        ColorDef::Ansi(ansi) => quote_spanned!(span=>Some(charmi::ColorDef::Ansi(#ansi))),
        ColorDef::Named(name) => {
            quote_spanned!(span=>Some(charmi::ColorDef::Named(#name.to_string())))
        },
        ColorDef::Rgb(r, g, b) => {
            quote_spanned!(span=>Some(charmi::ColorDef::Rgb(#r, #g, #b)))
        },
    }
}
