//! Proc-macros for deriving [`rimecraft_edcode`] traits.

use std::f32::consts::E;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Expr, Fields, Ident, Meta,
};

macro_rules! unsupported_object {
    ($tr:literal, $ty:literal) => {
        concat!("deriving `", $tr, "` to `", $ty, "` is not supported")
    };
}

macro_rules! fields_disallowed {
    () => {
        "variants with fields are not supported"
    };
}

macro_rules! discriminant_required {
    () => {
        "variants must have explicit discriminant"
    };
}

macro_rules! unsupported_repr {
    () => {
        "only `u*` and `i*` (excluding 128-bit types) repr is supported"
    };
}

macro_rules! repr_required {
    () => {
        "must specify repr"
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
                        macro_rules! ident_helper {
                            ($span:expr => $( $ty:ident ),*) => {
                                vec![
                                    $( Ident::new(stringify!($ty), $span) ),*
                                ]
                            };
                        }
                        let supported = iter.next().is_some_and(|x| {
                            if let TokenTree::Ident(id) = x {
                                if ident_helper!(span => u8, u16, u32, u64, i8, i16, i32, i64)
                                    .contains(&id)
                                {
                                    repr_type = Some(id);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                        if !supported {
                            return Error::new(span, unsupported_repr!())
                                .into_compile_error()
                                .into();
                        }
                    }
                }
            }
            let repr_type = match repr_type {
                None => {
                    return Error::new(ident.span(), repr_required!())
                        .into_compile_error()
                        .into()
                }
                Some(x) => x,
            };
            let expanded = quote! {
                impl ::rimecraft_edcode::Encode for #ident {
                    fn encode<B>(&self, mut buf: B) -> Result<(), std::io::Error>
                    where
                        B: ::rimecraft_edcode::bytes::BufMut,
                    {
                        let x:#repr_type = match self{
                            #( Self::#enum_idents => #enum_vals, )*
                        };
                        ::rimecraft_edcode::Encode::encode(&x, &mut buf)?;
                        Ok(())
                    }
                }
            };
            expanded.into()
        }
        Data::Struct(data) => syn::Error::new(
            data.struct_token.span,
            unsupported_object!("Encode", "struct"),
        )
        .to_compile_error()
        .into(),
        Data::Union(data) => Error::new(
            data.union_token.span,
            unsupported_object!("Encode", "union"),
        )
        .to_compile_error()
        .into(),
    }
}
