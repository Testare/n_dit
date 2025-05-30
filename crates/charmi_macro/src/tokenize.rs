use std::collections::HashMap;

use charmi_core::{CharCell, CharmiImage};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::{CharmiDynamic, CharmiDynamicKey, CharmiRepr, CharmiStatic};

impl ToTokens for CharmiRepr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Dyn(repr) => repr.to_tokens(tokens),
            Self::Stat(repr) => repr.to_tokens(tokens),
        }
    }
}

impl ToTokens for CharmiDynamic {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let map: HashMap<_, _> = self
            .0
            .iter()
            .map(|expr| (expr.key, expr.value.clone()))
            .collect();
        let none = quote!(None);

        let values = map
            .get(&CharmiDynamicKey::Values)
            .cloned()
            .or_else(|| {
                let m: Vec<_> = map
                    .iter()
                    .filter_map(|(key, value)| match key {
                        CharmiDynamicKey::VAttr => Some(quote!(values.attr = #value;)),
                        CharmiDynamicKey::VAttrKey(k) => {
                            Some(quote!(if let Some(v) = #value {
                                values.attr.get_or_insert_default().insert(#k, v);
                            }))
                        }
                        CharmiDynamicKey::VColors => Some(quote!(values.colors = #value;)),
                        CharmiDynamicKey::VColorKey(k) => {
                            Some(quote!(if let Some(v) = #value {
                                values.colors.get_or_insert_default().insert(#k, charmi::ColorDef::from(v));
                            }))
                        }
                        CharmiDynamicKey::VGap => Some(quote!(values.gap = #value;)),
                        CharmiDynamicKey::VSplit => Some(quote!(values.split = #value;)),
                        _ => None,
                    })
                    .collect();
                if m.is_empty() {
                    None
                } else {
                    Some(quote!({
                        let mut values = charmi::definition::Values {
                            gap: None,
                            colors: None,
                            attr: None,
                            split: None,
                        };
                        #(#m)*
                        Some(values)
                    }))
                }
            })
            .unwrap_or(quote!(None));
        let text = map.get(&CharmiDynamicKey::Text).unwrap_or(&none);
        let attr = map.get(&CharmiDynamicKey::Attr).unwrap_or(&none);
        let fg = map.get(&CharmiDynamicKey::Fg).unwrap_or(&none);
        let bg = map.get(&CharmiDynamicKey::Bg).unwrap_or(&none);
        let split = map.get(&CharmiDynamicKey::Split).unwrap_or(&none);

        quote!({
            let charmi_def = charmi::CharmiDef {
                fg: #fg,
                bg: #bg,
                attr: #attr,
                text: #text,
                split: #split,
                values: #values
            };
            charmi::CharmiImage::from(&charmi_def)
        })
        .to_tokens(tokens)
    }
}

impl ToTokens for CharmiStatic {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let image = CharmiImage::from(&self.0);

        let cells: Vec<u32> = image
            .cells()
            .iter()
            .flat_map(|&CharCell { ch, fg, bg, attr }| [ch, fg, bg, attr])
            .collect();
        let width = image.width32();
        let height = image.height32();

        // let width = image.width();
        // let height = image.height();
        // let text = self.0
        quote!(
            unsafe { charmi::CharmiImage::from_raw_u32(#width, #height, &[#(#cells),*])}
        )
        .to_tokens(tokens);
    }
}
