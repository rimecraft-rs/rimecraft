use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Lit, Meta, MetaNameValue, Path, parse_macro_input,
};

/// Entry point used by the crate root wrapper.
pub fn derive_localize_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_localize(&input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_localize(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let data = match &input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "#[derive(Localize)] can only be used on enums",
            ));
        }
    };

    // Parse optional enum-level prefix/suffix: #[localize(prefix = "...", suffix = "...")]
    let (enum_prefix, enum_suffix) = parse_enum_level_prefix_suffix(&input.attrs)?;

    let mut arms = Vec::new();

    for variant in &data.variants {
        let v_ident = &variant.ident;
        let segs = parse_variant_localize_segments(v_ident, &variant.attrs)?;
        let key = apply_prefix_suffix(&segs, enum_prefix.as_deref(), enum_suffix.as_deref());
        let key_lit = syn::LitStr::new(&key, v_ident.span());
        let pat_tokens = match &variant.fields {
            Fields::Unit => quote! { #ident::#v_ident },
            Fields::Unnamed(_) => quote! { #ident::#v_ident (..) },
            Fields::Named(_) => quote! { #ident::#v_ident { .. } },
        };
        arms.push(quote! { #pat_tokens => ::std::borrow::Cow::Borrowed(#key_lit), });
    }

    let expanded = quote! {
        impl rimecraft_text::Localize for #ident {
            fn localization_key(&self) -> ::std::borrow::Cow<'_, str> {
                match self {
                    #( #arms )*
                }
            }
        }
    };

    Ok(expanded.into())
}

/// Parse enum-level `#[localize(prefix = "...", suffix = "...")]`.
fn parse_enum_level_prefix_suffix(
    attrs: &[Attribute],
) -> syn::Result<(Option<String>, Option<String>)> {
    let mut prefix: Option<String> = None;
    let mut suffix: Option<String> = None;

    for attr in attrs {
        if !attr.path().is_ident("localize") {
            continue;
        }

        if let Meta::List(list) = &attr.meta {
            // Try parse name-value pairs: prefix = "...", suffix = "..."
            let pairs =
                syn::punctuated::Punctuated::<MetaNameValue, syn::Token![,]>::parse_terminated
                    .parse2(list.tokens.clone());

            if let Ok(pairs) = pairs {
                for nv in pairs {
                    if nv.path.is_ident("prefix") {
                        if let Expr::Lit(el) = nv.value {
                            if let Lit::Str(s) = el.lit {
                                prefix = Some(s.value());
                            } else {
                                return Err(syn::Error::new(
                                    el.lit.span(),
                                    "prefix must be a string literal",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new(
                                nv.value.span(),
                                "prefix must be a string literal",
                            ));
                        }
                    } else if nv.path.is_ident("suffix") {
                        if let Expr::Lit(el) = nv.value {
                            if let Lit::Str(s) = el.lit {
                                suffix = Some(s.value());
                            } else {
                                return Err(syn::Error::new(
                                    el.lit.span(),
                                    "suffix must be a string literal",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new(
                                nv.value.span(),
                                "suffix must be a string literal",
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok((prefix, suffix))
}

fn parse_variant_localize_segments(
    v_ident: &syn::Ident,
    attrs: &[Attribute],
) -> syn::Result<Vec<String>> {
    // Find #[localize(...)] / #[localize]
    let mut segments: Option<Vec<String>> = None;
    for attr in attrs {
        if !attr.path().is_ident("localize") {
            continue;
        }

        match &attr.meta {
            Meta::List(list) => {
                let mut segs = Vec::new();
                // Parse tokens inside (...) as comma-separated identifiers or string literals
                let exprs = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
                    .parse2(list.tokens.clone())?;
                for expr in exprs {
                    match expr {
                        Expr::Path(p) => {
                            let s = path_to_ident_str(&p.path);
                            segs.push(s);
                        }
                        Expr::Infer(_) => {
                            // Handle underscore `_` as a placeholder for variant name
                            segs.push("_".to_owned());
                        }
                        Expr::Lit(el) => {
                            if let Lit::Str(s) = el.lit {
                                segs.push(s.value());
                            } else {
                                return Err(syn::Error::new(
                                    el.lit.span(),
                                    "Only string literals are supported in #[localize(...)]",
                                ));
                            }
                        }
                        _ => {
                            return Err(syn::Error::new(
                                expr.span(),
                                "Only identifiers, underscores, and string literals are supported in #[localize(...)]",
                            ));
                        }
                    }
                }
                segments = Some(segs);
            }
            Meta::Path(_) => {
                // #[localize] with no args — same as #[localize(_)]
                segments = Some(vec!["_".to_owned()]);
            }
            Meta::NameValue(nv) => {
                // Support: #[localize = "a.b._.c"]
                match &nv.value {
                    Expr::Lit(elit) => {
                        if let Lit::Str(s) = &elit.lit {
                            segments = Some(
                                s.value()
                                    .split('.')
                                    .filter(|p| !p.is_empty())
                                    .map(|p| p.to_owned())
                                    .collect(),
                            );
                        } else {
                            return Err(syn::Error::new(
                                elit.lit.span(),
                                "#[localize = \"a.b._.c\"] expected a string literal",
                            ));
                        }
                    }
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            "#[localize = ...] expects a string literal like \"a.b._.c\" or use #[localize(...)]",
                        ));
                    }
                }
            }
        }
    }

    let mut segs = segments.unwrap_or_else(|| vec!["_".to_owned()]);

    // Replace '_' with snake_case of variant name
    for seg in &mut segs {
        if seg == "_" {
            *seg = to_snake_case_ident(&v_ident.to_string());
        }
    }

    Ok(segs)
}

/// Apply optional prefix/suffix to the generated key segments.
fn apply_prefix_suffix(segs: &[String], prefix: Option<&str>, suffix: Option<&str>) -> String {
    let mut out: Vec<&str> = Vec::with_capacity(segs.len() + 2);
    if let Some(p) = prefix {
        for part in p.split('.') {
            if !part.is_empty() {
                out.push(part);
            }
        }
    }
    for s in segs {
        out.push(s.as_str());
    }
    if let Some(suf) = suffix {
        for part in suf.split('.') {
            if !part.is_empty() {
                out.push(part);
            }
        }
    }
    out.join(".")
}

fn path_to_ident_str(p: &Path) -> String {
    p.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Convert a Rust-style PascalCase or camelCase identifier into snake_case.
fn to_snake_case_ident(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let prev_is_lower =
                i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            if i > 0 && (prev_is_lower || (next_is_lower && chars[i - 1].is_uppercase())) {
                out.push('_');
            }

            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else if ch == '-' {
            out.push('_');
        } else if ch.is_alphanumeric() || ch == '_' {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case_ident() {
        assert_eq!(to_snake_case_ident("MyCase"), "my_case");
        assert_eq!(to_snake_case_ident("HTTPServer"), "http_server");
        assert_eq!(to_snake_case_ident("someValue"), "some_value");
        assert_eq!(to_snake_case_ident("ABC"), "abc");
        assert_eq!(
            to_snake_case_ident("already_snake_case"),
            "already_snake_case"
        );
        assert_eq!(to_snake_case_ident("IOError"), "io_error");
        assert_eq!(to_snake_case_ident("First"), "first");
    }
}
