//! Proc-macros for deriving [`rimecraft_edcode`] traits.

use proc_macro::TokenStream;

/// Derive [`rimecraft_edcode::Encode`] to types.
#[proc_macro_derive(Encode)]
pub fn derive_encode(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
