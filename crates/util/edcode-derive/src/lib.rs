//! Proc-macros for deriving [`rimecraft_edcode`] traits.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Expr, Fields, Ident, Meta,
};

macro_rules! unsupported_error {
    ($tr:literal, $ty:literal) => {
        concat!("deriving `", $tr, "` to `", $ty, "` is not supported")
    };
}

macro_rules! fields_disallowed {
    () => {
        "enum with fields is not supported"
    };
}

macro_rules! discriminant_required {
    () => {
        "enum variants must have explicit discriminant"
    };
}

/// Derive [`rimecraft_edcode::Encode`] to objects.
#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Enum(data) => {
            let ident = input.ident;
            let mut enum_idents: Vec<Ident> = vec![];
            let mut enum_vals: Vec<Expr> = vec![];
            for var in data.variants.into_iter() {
                if let Fields::Unit = var.fields {
                    enum_idents.push(var.ident.clone());
                } else {
                    return Error::new(var.fields.span(), fields_disallowed!())
                        .into_compile_error()
                        .into();
                }
                let has_disc_err = !var.discriminant.clone().is_some_and(|(_, e)| {
                    enum_vals.push(e);
                    true
                });
                if has_disc_err {
                    return Error::new(var.span(), discriminant_required!())
                        .into_compile_error()
                        .into();
                }
            }
            let mut repr_type: Option<Ident> = None;
            for attr in input.attrs {
                if let Meta::List(meta) = attr.meta {
                    let is_repr = meta
                        .path
                        .require_ident()
                        .is_ok_and(|id| id == &Ident::new("repr", id.span()));
                    if is_repr {
                        let mut iter = meta.tokens.into_iter().peekable();
                        let span = iter.peek().span();
                        todo!()
                    }
                }
            }
            let expanded = quote! {
                impl Encode for #ident {
                    fn encode<B>(&self, mut buf: B) -> Result<(), std::io::Error>
                    where
                        B: ::rimecraft_edcode::bytes::BufMut,
                    {
                        let x = match self{
                            #( Self::#enum_idents => #enum_vals, )*
                        };
                        x.encode(&mut buf)?;
                        Ok(())
                    }
                }
            };
            expanded.into()
        }
        Data::Struct(data) => syn::Error::new(
            data.struct_token.span,
            unsupported_error!("Encode", "struct"),
        )
        .to_compile_error()
        .into(),
        Data::Union(data) => {
            Error::new(data.union_token.span, unsupported_error!("Encode", "union"))
                .to_compile_error()
                .into()
        }
    }
}
