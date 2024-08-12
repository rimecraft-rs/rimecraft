//! Proc-macros for deriving `rimecraft_edcode2` traits.
//!
//! __You shouldn't use this crate directly__, use `rimecraft_edcode2` crate
//! with `derive` feature flag instead.

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DataEnum, DeriveInput, Error, Expr,
    Fields, Ident, Meta,
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
        "only primitive reprs which can be safely turned into an `i32` are supported"
    };
}

macro_rules! repr_required {
    () => {
        "must specify repr"
    };
}

/// Common parsing code for deriving to `enum`.
fn parse_derive_enum(
    ident: Ident,
    attrs: Vec<Attribute>,
    data: DataEnum,
) -> Result<(Ident, Ident, Vec<Ident>, Vec<Expr>), TokenStream> {
    let mut enum_idents: Vec<Ident> = vec![];
    let mut enum_vals: Vec<Expr> = vec![];
    for var in data.variants.into_iter() {
        if matches!(var.fields, Fields::Unit) {
            enum_idents.push(var.ident.clone());
        } else {
            return Err(Error::new(var.fields.span(), fields_disallowed!())
                .into_compile_error()
                .into());
        }
        let has_disc_err = !var.discriminant.clone().is_some_and(|(_, e)| {
            enum_vals.push(e);
            true
        });
        if has_disc_err {
            return Err(Error::new(var.span(), discriminant_required!())
                .into_compile_error()
                .into());
        }
    }
    let mut repr_type: Option<Ident> = None;
    for attr in attrs {
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
                        if ident_helper!(span => u8, u16, i8, i16, i32).contains(&id) {
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
                    return Err(Error::new(span, unsupported_repr!())
                        .into_compile_error()
                        .into());
                }
            }
        }
    }
    let repr_type = repr_type.ok_or_else(|| {
        Into::<TokenStream>::into(Error::new(ident.span(), repr_required!()).into_compile_error())
    })?;
    Ok((ident, repr_type, enum_idents, enum_vals))
}

/// Derive `rimecraft_edcode2::Encode` to objects.
///
/// # Enum
///
/// ## Requirements:
/// - All variants must be field-less.
/// - Enum must explicitly specify its representation through `#[repr()]`, and
///   only primitive representations which can be safely turned into an `i32`
///   are allowed.
/// - All variants must have explicit discriminant.
#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Enum(data) => {
            let (ident, repr_type, enum_idents, enum_vals) =
                match parse_derive_enum(input.ident, input.attrs, data) {
                    Ok(x) => x,
                    Err(err) => return err,
                };
            let expanded = quote! {
                impl<B: ::rimecraft_edcode2::BufMut> ::rimecraft_edcode2::Encode<B> for #ident {
                    fn encode(&self, mut buf: B) -> Result<(), ::rimecraft_edcode2::BoxedError<'static>> {
                        let x:#repr_type = match self {
                            #( Self::#enum_idents => #enum_vals, )*
                        };
                        ::rimecraft_edcode2::Encode::encode(&x, &mut buf)?;
                        Ok(())
                    }
                }
            };
            expanded.into()
        }
        Data::Struct(data) => Error::new(
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

/// Derive `rimecraft_edcode2::Decode` to objects.
#[proc_macro_derive(Decode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Enum(data) => {
            let (ident, repr_type, enum_idents, enum_vals) =
                match parse_derive_enum(input.ident, input.attrs, data) {
                    Ok(x) => x,
                    Err(err) => {
                        return err;
                    }
                };
            let expanded = quote! {
                impl<'de, B: ::rimecraft_edcode2::Buf> ::rimecraft_edcode2::Decode<'de, B> for #ident {
                    fn decode(mut buf: B) -> Result<Self, ::rimecraft_edcode2::BoxedError<'de>> {
                        let x:#repr_type = ::rimecraft_edcode2::Decode::decode(&mut buf)?;
                        let var = match x {
                            #( #enum_vals => Self::#enum_idents, )*
                            unknown => return Err(Box::new(std::io::Error::other(format!("unknown variant {}", unknown)))),
                        };
                        Ok(var)
                    }
                }
            };
            expanded.into()
        }
        Data::Struct(data) => Error::new(
            data.struct_token.span,
            unsupported_object!("Decode", "struct"),
        )
        .to_compile_error()
        .into(),
        Data::Union(data) => Error::new(
            data.union_token.span,
            unsupported_object!("Decode", "union"),
        )
        .into_compile_error()
        .into(),
    }
}
