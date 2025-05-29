use charmi_core::CharmiDef;
use quote::quote_spanned;
use std::str::FromStr;
use syn::{
    Error, Expr, ExprAssign, ExprField, ExprPath, ExprTry, LitStr, Member, Token, parse::Parse,
    spanned::Spanned,
};
use toml::Table;
use unicode_width::UnicodeWidthStr;

use crate::{
    CharmiDynamic, CharmiDynamicExpr, CharmiDynamicKey, CharmiRepr, CharmiStatic, conv, validate,
};

impl Parse for CharmiRepr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str: LitStr = input.parse()?;
        let clean_str = clean_input_str(&lit_str.value());
        let table: Table = Table::from_str(&clean_str).map_err(|e| {
            Error::new(
                lit_str.span(),
                format!("Invalid TOML {:?}: {}", e.span(), e.message()),
            )
        })?;
        crate::validate::validate_charmi_toml(lit_str.span(), &table)?;
        let def: CharmiDef = table.try_into().map_err(|e| {
            Error::new(
                lit_str.span(),
                format!(
                    "Cannot parse CharmiImage from table {:?}: {}",
                    e.span(),
                    e.message()
                ),
            )
        })?;
        // optional comma
        if input.peek(Token![,]) {
            _ = input.parse::<Token![,]>();
        }
        let extra_dyn_expr: Vec<_> = input
            .parse_terminated(CharmiDynamicExpr::parse, Token![,])?
            .into_iter()
            .collect();

        validate::validate_dyn_expr(&def, &extra_dyn_expr)?;

        if extra_dyn_expr.is_empty() {
            Ok(Self::Stat(CharmiStatic(def)))
        } else {
            let mut dyn_expr = conv::def_to_dyn_expr(lit_str.span(), def);
            dyn_expr.extend(extra_dyn_expr);
            Ok(Self::Dyn(CharmiDynamic(dyn_expr)))
        }
    }
}

impl Parse for CharmiDynamicExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let try_assign = input.fork();
        let try_single_path = input.fork();
        let try_single_path_as_try = input.fork();

        let original_parse = try_assign.parse::<ExprAssign>();

        if let Ok(assignment) = original_parse {
            _ = input.parse::<ExprAssign>();
            let span = assignment.span();
            let value = *assignment.right;
            let expr_span = value.span();
            let (key_expr, value) = if let Expr::Try(expr_try) = *assignment.left {
                (expr_try.expr, quote_spanned!(expr_span=> #value))
            } else {
                (assignment.left, quote_spanned!(expr_span=> Some(#value)))
            };
            return Ok(Self {
                span,
                value,
                // TODO parse key as potentially optional
                key: CharmiDynamicKey::try_from(*key_expr)?,
            });
        }

        if let Ok(tryexpr) = try_single_path_as_try.parse::<ExprTry>() {
            _ = input.parse::<ExprTry>();
            let span = tryexpr.span();
            if let Expr::Path(path) = *tryexpr.expr {
                return Ok(Self {
                    span,
                    key: CharmiDynamicKey::try_from(Expr::Path(path.clone()))?,
                    value: quote_spanned!(span=>#path),
                });
            }
        } else if let Ok(path) = try_single_path.parse::<syn::ExprPath>() {
            _ = input.parse::<ExprPath>();
            return Ok(Self {
                span: path.span(),
                key: CharmiDynamicKey::try_from(Expr::Path(path.clone()))?,
                value: quote_spanned!(path.span()=>Some(#path)),
            });
        };

        _ = input.parse::<ExprAssign>();
        original_parse.map(|_| unreachable!("If this was okay it would've returned earlier"))
    }
}

impl TryFrom<Expr> for CharmiDynamicKey {
    type Error = Error;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        match expr {
            Expr::Field(field) => {
                let (first, second) = clean_field(&field)?;
                match (first.as_str(), second.as_str()) {
                    ("values", "attr") => Ok(CharmiDynamicKey::VAttr),
                    ("values", "colors") => Ok(CharmiDynamicKey::VColors),
                    ("values", "gap") => Ok(CharmiDynamicKey::VGap),
                    ("values", "split") => Ok(CharmiDynamicKey::VSplit),
                    ("values.attr", chstr) => {
                        let mut chs = chstr.chars();
                        let ch = chs.next();
                        if chstr.width() == 1 && ch.is_some() && chs.next().is_none() {
                            Ok(CharmiDynamicKey::VAttrKey(ch.unwrap()))
                        } else {
                            Err(Error::new(
                                field.member.span(),
                                "attr key must be a single-cell char",
                            ))
                        }
                    },
                    ("values.colors", chstr) => {
                        let mut chs = chstr.chars();
                        let ch = chs.next();
                        if chstr.width() == 1 && ch.is_some() && chs.next().is_none() {
                            Ok(CharmiDynamicKey::VColorKey(ch.unwrap()))
                        } else {
                            Err(Error::new(
                                field.member.span(),
                                "colors key must be a single-cell char",
                            ))
                        }
                    },
                    _ => Err(Error::new(field.span(), "Unrecognizable key")),
                }
            },
            Expr::Path(path) => match clean_path(&path)?.as_str() {
                "attr" => Ok(CharmiDynamicKey::Attr),
                "bg" => Ok(CharmiDynamicKey::Bg),
                "fg" => Ok(CharmiDynamicKey::Fg),
                "split" => Ok(CharmiDynamicKey::Split),
                "text" => Ok(CharmiDynamicKey::Text),
                "values" => Ok(CharmiDynamicKey::Values),
                _ => Err(Error::new(path.span(), "Unrecognizable key")),
            },
            _ => Err(Error::new(
                expr.span(),
                "Unexpected expr type for charmi key",
            )),
        }
    }
}

fn clean_field(expr: &ExprField) -> Result<(String, String), Error> {
    let ExprField {
        attrs,
        base,
        member,
        ..
    } = expr;
    if !attrs.is_empty() {
        return Err(Error::new(attrs[0].span(), "Unexpected attribute"));
    }
    let first_part = match base.as_ref() {
        Expr::Path(path) => clean_path(path)?,
        Expr::Field(field) => {
            let (s1, s2) = clean_field(field)?;
            format!("{s1}.{s2}")
        },
        other => return Err(Error::new(other.span(), "Unexpected expression")),
    };
    let second_part = match member {
        Member::Named(ident) => ident.to_string(),
        Member::Unnamed(idx) => format!("{}", idx.index),
    };
    Ok((first_part, second_part))
}

fn clean_path(expr: &ExprPath) -> Result<String, Error> {
    let ExprPath { attrs, qself, path } = expr;
    if !attrs.is_empty() {
        return Err(Error::new(attrs[0].span(), "Unexpected attribute"));
    }
    if let Some(qself) = qself {
        return Err(Error::new(qself.span(), "Unexpected self modifier"));
    }
    let Some(ident) = path.get_ident() else {
        return Err(Error::new(path.span(), "Unrecognizable key"));
    };
    Ok(ident.to_string())
}

fn clean_input_str(s: &str) -> String {
    let lines: Vec<_> = s
        .lines()
        .rev()
        .skip_while(|s| s.trim().is_empty())
        .collect();
    let lines: Vec<_> = lines
        .into_iter()
        .rev()
        .skip_while(|s| s.trim().is_empty())
        .collect();
    let leading_whitespace = lines
        .iter()
        .map(|s| s.len() - s.trim_start().len())
        .min()
        .unwrap_or(0);
    let lines: Vec<_> = lines
        .into_iter()
        .map(|s| &s[leading_whitespace..])
        .collect();
    lines.join("\n")
}
