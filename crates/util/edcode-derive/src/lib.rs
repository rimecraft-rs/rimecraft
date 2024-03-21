//! Proc-macros for deriving [`rimecraft_edcode`] traits.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ parse_macro_input, Data, DeriveInput};

macro_rules! unsupported_error {
    ($tr:literal, $ty:literal) => {
        concat!("deriving `", $tr, "` to `", $ty, "` is not supported")
    };
}

/// Derive [`rimecraft_edcode::Encode`] to objects.
#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Enum(data) => {
            // TODO: derive `Encode` for `enum`.
            let expanded=quote!{
                impl Encode for Foo {
                    fn encode<B>(&self, buf: B) -> Result<(), std::io::Error>
                    where
                        B: rimecraft_edcode::bytes::BufMut,
                    {
                    }
                }
            };
            todo!()
        }
        Data::Struct(data) => syn::Error::new(
            data.struct_token.span,
            unsupported_error!("Encode", "struct"),
        )
        .to_compile_error()
        .into(),
        Data::Union(data) => {
            syn::Error::new(data.union_token.span, unsupported_error!("Encode", "union"))
                .to_compile_error()
                .into()
        }
    }
}
