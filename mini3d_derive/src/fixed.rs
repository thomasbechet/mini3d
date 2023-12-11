use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, Lit};

fn parse_expr_literal(expr: &Expr) -> Option<(bool, &str, &str)> {
    fn parse_lit(lit: &ExprLit) -> Option<(&str, &str)> {
        match &lit.lit {
            Lit::Int(lit) => Some((lit.suffix(), lit.base10_digits())),
            Lit::Float(lit) => Some((lit.suffix(), lit.base10_digits())),
            _ => None,
        }
    }
    match expr {
        Expr::Lit(lit) => {
            parse_lit(lit).map(|(suffix, lit_nosuffix)| (false, suffix, lit_nosuffix))
        }
        Expr::Unary(unary) => match unary.op {
            syn::UnOp::Neg(_) => match unary.expr.as_ref() {
                Expr::Lit(lit) => {
                    parse_lit(lit).map(|(suffix, lit_nosuffix)| (true, suffix, lit_nosuffix))
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

pub fn convert_fixed(input: &Expr) -> TokenStream {
    let (signed, suffix, digits) = parse_expr_literal(input).unwrap();
    let digits = if signed {
        format!("-{}", digits)
    } else {
        digits.to_string()
    };
    match suffix {
        "u32" => quote! { U32::lit(#digits) },
        "u32f8" => quote! { U32F8::lit(#digits) },
        "u32f16" => quote! { U32F16::lit(#digits) },
        "u32f24" => quote! { U32F24::lit(#digits) },
        "i32" => quote! { I32::lit(#digits) },
        "i32f8" => quote! { I32F8::lit(#digits) },
        "i32f16" => quote! { I32F16::lit(#digits) },
        "i32f24" => quote! { I32F24::lit(#digits) },
        _ => quote! { #digits.into() },
    }
    .into()
}
