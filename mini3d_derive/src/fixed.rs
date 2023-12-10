use syn::{
    parse_quote,
    visit_mut::{self, VisitMut},
    Expr, ExprLit, Lit,
};

pub(crate) struct FixedPointLiteralReplacer;

impl VisitMut for FixedPointLiteralReplacer {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        if let Expr::Lit(ExprLit { lit, .. }) = i {
            let suffix_lit = match lit {
                Lit::Int(lit) => Some((lit.suffix(), lit.base10_digits())),
                Lit::Float(lit) => Some((lit.suffix(), lit.base10_digits())),
                _ => None,
            };
            if let Some((suffix, lit_nosuffix)) = suffix_lit {
                match suffix {
                    "u32" => *i = parse_quote! { U32::lit(#lit_nosuffix) },
                    "u32f8" => *i = parse_quote! { U32F8::lit(#lit_nosuffix) },
                    "u32f16" => *i = parse_quote! { U32F16::lit(#lit_nosuffix) },
                    "u32f24" => *i = parse_quote! { U32F24::lit(#lit_nosuffix) },
                    "i32" => *i = parse_quote! { I32::lit(#lit_nosuffix) },
                    "i32f8" => *i = parse_quote! { I32F8::lit(#lit_nosuffix) },
                    "i32f16" => *i = parse_quote! { I32F16::lit(#lit_nosuffix) },
                    "i32f24" => *i = parse_quote! { I32F24::lit(#lit_nosuffix) },
                    _ => *i = parse_quote! { #lit_nosuffix.into() },
                }
            }
        } else {
            visit_mut::visit_expr_mut(self, i)
        }
    }
}
