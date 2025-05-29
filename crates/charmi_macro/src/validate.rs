use charmi_core::{CharmiDef, ColorDef};
use proc_macro2::Span;
use std::collections::HashMap;
use toml::{Table, Value};
use unicode_width::UnicodeWidthStr;

use crate::{CharmiDynamicExpr, CharmiDynamicKey, CharmiError};

pub fn validate_charmi_toml(span: Span, table: &Table) -> syn::Result<()> {
    table
        .iter()
        .flat_map(validate_charmi_kv)
        .sum::<CharmiError>()
        .none_if_empty()
        .map(|e| e.to_syn_error(span))
        .unwrap_or(Ok(()))
}

fn validate_charmi_values_kv((key, value): (&String, &Value)) -> Option<CharmiError> {
    let error_key = format!("values.{key}");
    match key.as_str() {
        "gap" => {
            if let Some(gapch) = value.as_str() {
                if gapch.width() != 1 || gapch.chars().count() != 1 {
                    // String should be a single character one cell wide
                    // In the future we can disambiguate these errors
                    CharmiError::char_length_error(&error_key, 1)
                } else {
                    None
                }
            } else {
                CharmiError::wrong_type(&error_key, "string", value.type_str())
            }
        },
        "colors" => {
            let Some(color_map) = value.as_table() else {
                return CharmiError::wrong_type(&error_key, "table", value.type_str());
            };
            color_map
                .iter()
                .map(|(k, v)| {
                    let error_key = format!("{error_key}.{k}");
                    let e1 = if k.width() != 1 || k.chars().count() != 1 {
                        CharmiError::char_length_error(&error_key, 1)
                    } else {
                        None
                    }
                    .unwrap_or_default();

                    let e2 = if let Some(color) = v.as_str() {
                        if ColorDef::recognized_color(color) {
                            None
                        } else {
                            CharmiError::unrecognized_color(color)
                        }
                    } else if !v.is_integer()
                        && (v
                            .as_array()
                            .map(|a| a.len() != 3 && a.iter().any(|av| !av.is_integer()))
                            .unwrap_or(true))
                    {
                        CharmiError::wrong_type(
                            &error_key,
                            "string, integer, or array of 3 integers",
                            v.type_str(),
                        )
                    } else {
                        None
                    }
                    .unwrap_or_default();

                    e1 + e2
                })
                .sum::<CharmiError>()
                .none_if_empty()
        },
        "attr" => {
            let Some(attr_map) = value.as_table() else {
                return CharmiError::wrong_type(&error_key, "table", value.type_str());
            };
            attr_map
                .iter()
                .map(|(k, v)| {
                    let error_key = format!("{error_key}.{k}");
                    let e1 = if k.width() != 1 || k.chars().count() != 1 {
                        CharmiError::char_length_error(&error_key, 1)
                    } else {
                        None
                    }
                    .unwrap_or_default();

                    let e2 = if let Some(attr_name) = v.as_str() {
                        if ["underline", "bold"].contains(&attr_name) {
                            None
                        } else {
                            CharmiError::unrecognized_attr(attr_name)
                        }
                    } else {
                        CharmiError::wrong_type(&error_key, "string", v.type_str())
                    }
                    .unwrap_or_default();

                    e1 + e2
                })
                .sum::<CharmiError>()
                .none_if_empty()
        },
        "split" => {
            if let Some(split) = value.as_str() {
                if split.width() != 2 || split.chars().count() != 2 {
                    CharmiError::char_length_error(&error_key, 2)
                } else {
                    None
                }
            } else {
                CharmiError::wrong_type(&error_key, "string", value.type_str())
            }
        },
        _ => CharmiError::unexpected_key(error_key.as_str()),
    }
}

fn validate_charmi_kv((key, value): (&String, &Value)) -> Option<CharmiError> {
    match key.as_str() {
        "text" | "fg" | "bg" | "attr" => {
            if !value.is_str() {
                // CharmiError::wrong_type(key, "string", value.type_str())
                None
            } else {
                None
            }
        },
        "splits" => None,
        "values" => {
            if let Some(value_table) = value.as_table() {
                value_table
                    .iter()
                    .flat_map(validate_charmi_values_kv)
                    .sum::<CharmiError>()
                    .none_if_empty()
            } else {
                CharmiError::wrong_type(key, "table", value.type_str())
            }
        },
        _ => CharmiError::unexpected_key(key),
    }
}

pub fn validate_dyn_expr(
    charmi_def: &CharmiDef,
    dyn_expr: &[CharmiDynamicExpr],
) -> syn::Result<()> {
    let mut index_map: HashMap<CharmiDynamicKey, Vec<Span>> = HashMap::new();
    if charmi_def.bg.is_some() {
        index_map.insert(CharmiDynamicKey::Bg, Default::default());
    }
    if charmi_def.fg.is_some() {
        index_map.insert(CharmiDynamicKey::Fg, Default::default());
    }
    if charmi_def.text.is_some() {
        index_map.insert(CharmiDynamicKey::Text, Default::default());
    }
    if charmi_def.attr.is_some() {
        index_map.insert(CharmiDynamicKey::Attr, Default::default());
    }
    if let Some(values) = charmi_def.values.as_ref() {
        if values.gap.is_some() {
            index_map.insert(CharmiDynamicKey::VGap, Default::default());
        }
        // TODO Split not defined yet
        /*if values.split.is_some() {
            index_map.insert(CharmiDynamicKey::VSplit, Default::default());
        }*/
        if let Some(attr) = values.attr.as_ref() {
            index_map.extend(
                attr.keys()
                    .map(|k| (CharmiDynamicKey::VAttrKey(*k), Default::default())),
            );
        }
        if let Some(colors) = values.colors.as_ref() {
            index_map.extend(
                colors
                    .keys()
                    .map(|k| (CharmiDynamicKey::VColorKey(*k), Default::default())),
            );
        }
    }
    for de in dyn_expr.iter() {
        index_map
            .entry(de.key)
            .and_modify(|v| v.push(de.span))
            .or_default();
    }
    use CharmiDynamicKey::*;
    dyn_expr
        .iter()
        .flat_map(|expr| match expr.key {
            Values => charmi_def
                .values
                .is_some()
                .then(|| syn::Error::new(expr.span, "Cannot redefine values")),
            VGap | VSplit => index_map.contains_key(&Values).then(|| {
                syn::Error::new(expr.span, "Cannot define this key when `values` is defined")
            }),
            VColors => {
                if charmi_def.values.is_some()
                    && charmi_def.values.as_ref().unwrap().colors.is_some()
                {
                    Some(syn::Error::new(expr.span, "Cannot redefine values.colors"))
                } else if index_map.contains_key(&Values) {
                    Some(syn::Error::new(
                        expr.span,
                        "Cannot define this key when `values` is defined",
                    ))
                } else {
                    None
                }
            },
            VAttr => {
                if charmi_def.values.is_some() && charmi_def.values.as_ref().unwrap().attr.is_some()
                {
                    Some(syn::Error::new(expr.span, "Cannot redefine values.attr"))
                } else if index_map.contains_key(&Values) {
                    Some(syn::Error::new(
                        expr.span,
                        "Cannot define this key when `values` is defined",
                    ))
                } else {
                    None
                }
            },
            VAttrKey(_) => index_map
                .contains_key(&Values)
                .then(|| {
                    syn::Error::new(expr.span, "Cannot define this key when `values` is defined")
                })
                .or(index_map.contains_key(&VAttr).then(|| {
                    syn::Error::new(
                        expr.span,
                        "Cannot define this key when `values.attr` is defined",
                    )
                })),
            VColorKey(_) => index_map
                .contains_key(&Values)
                .then(|| {
                    syn::Error::new(expr.span, "Cannot define this key when `values` is defined")
                })
                .or(index_map.contains_key(&VColors).then(|| {
                    syn::Error::new(
                        expr.span,
                        "Cannot define this key when `values.colors` is defined",
                    )
                })),
            _ => None,
        })
        .chain(
            index_map
                .values()
                .flatten()
                .map(|span| syn::Error::new(*span, "duplicate key")),
        )
        .reduce(|e1, _e2| {
            // e1.combine(e2); // TODO figure out why this crashes
            e1
        })
        .map(Err)
        .unwrap_or(Ok(()))
}
