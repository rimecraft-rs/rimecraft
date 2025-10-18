//! Derive macros for rimecraft-text.
//!
//! This crate provides procedural macros for automatically implementing the
//! [`rimecraft_text::Localize`] trait on enums.
//!
//! # Examples
//!
//! Basic usage with `#[localize]` attribute:
//!
//! ```
//! use rimecraft_text::Localize;
//!
//! #[derive(Localize)]
//! enum GameMode {
//!     #[localize(options, generic, gameMode, survival)]
//!     Survival,
//!
//!     #[localize(options, generic, gameMode, creative)]
//!     Creative,
//!
//!     #[localize(options, generic, gameMode, adventure)]
//!     Adventure,
//! }
//!
//! assert_eq!(
//!     GameMode::Survival.localization_key(),
//!     "options.generic.gameMode.survival"
//! );
//! ```
//!
//! Using `_` placeholder for variant name (converted to snake_case):
//!
//! ```
//! use rimecraft_text::Localize;
//!
//! #[derive(Localize)]
//! enum Difficulty {
//!     #[localize(options, difficulty, _)]
//!     Peaceful,
//!
//!     #[localize(options, difficulty, _)]
//!     Easy,
//!
//!     #[localize(options, difficulty, _)]
//!     Normal,
//!
//!     #[localize(options, difficulty, _)]
//!     Hard,
//! }
//!
//! assert_eq!(Difficulty::Peaceful.localization_key(), "options.difficulty.peaceful");
//! assert_eq!(Difficulty::Hard.localization_key(), "options.difficulty.hard");
//! ```
//!
//! Default behavior without explicit attribute (uses variant name):
//!
//! ```
//! use rimecraft_text::Localize;
//!
//! #[derive(Localize)]
//! enum SimpleEnum {
//!     FirstVariant,   // => "first_variant"
//!     SecondVariant,  // => "second_variant"
//! }
//!
//! assert_eq!(SimpleEnum::FirstVariant.localization_key(), "first_variant");
//! ```
//!
//! String literal syntax:
//!
//! ```
//! use rimecraft_text::Localize;
//!
//! #[derive(Localize)]
//! enum Status {
//!     #[localize = "status.online"]
//!     Online,
//!
//!     #[localize = "status.away._.suffix"]
//!     Away,  // => "status.away.away.suffix"
//! }
//!
//! assert_eq!(Status::Online.localization_key(), "status.online");
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Lit, Meta, Path, parse::Parser, parse_macro_input,
    spanned::Spanned,
};

/// Derives the `Localize` trait for enums.
///
/// This macro generates an implementation of `rimecraft_text::Localize` for enum types.
/// Each variant can have a `#[localize(...)]` attribute to specify its localization key segments.
///
/// # Attributes
///
/// - `#[localize(seg1, seg2, ...)]` - Segments joined with `.`. Use `_` for variant name in snake_case.
/// - `#[localize]` - Uses variant name in snake_case as the key.
/// - `#[localize = "dot.separated.key"]` - String literal with `.` separators, `_` expands to variant name.
///
/// # Examples
///
/// ```
/// use rimecraft_text::Localize;
///
/// #[derive(Localize)]
/// enum ParticlesMode {
///     #[localize(options, particles, _)]
///     All,
///
///     #[localize(options, particles, _)]
///     Decreased,
///
///     #[localize(options, particles, _)]
///     Minimal,
/// }
///
/// assert_eq!(ParticlesMode::All.localization_key(), "options.particles.all");
/// ```
#[proc_macro_derive(Localize, attributes(localize))]
pub fn derive_localize(input: TokenStream) -> TokenStream {
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

    let mut arms = Vec::new();

    for variant in &data.variants {
        let v_ident = &variant.ident;
        let segs = parse_variant_localize_segments(v_ident, &variant.attrs)?;
        let key = segs.join(".");
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

fn path_to_ident_str(p: &Path) -> String {
    p.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Convert a Rust-style PascalCase or camelCase identifier into snake_case.
///
/// This is intentionally simple and avoids extra dependencies inside proc-macro.
///
/// # Examples
///
/// - `MyCase` → `my_case`
/// - `HTTPServer` → `http_server`
/// - `someValue` → `some_value`
/// - `IOError` → `io_error`
fn to_snake_case_ident(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            // Add underscore before uppercase letter if:
            // 1. Not at the start
            // 2. Previous char was lowercase or digit
            // 3. OR next char is lowercase (handles "HTTPServer" -> "http_server")
            let prev_is_lower =
                i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            if i > 0 && (prev_is_lower || (next_is_lower && chars[i - 1].is_uppercase())) {
                out.push('_');
            }

            // Convert to lowercase
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else if ch == '-' {
            // Normalize dashes to underscores
            out.push('_');
        } else if ch.is_alphanumeric() || ch == '_' {
            out.push(ch);
        }
        // Skip other punctuation
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
