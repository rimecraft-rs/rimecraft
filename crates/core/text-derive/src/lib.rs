//! Procedural macros for deriving text-related traits.

mod derive_localize;

use proc_macro::TokenStream;

/// Derives `rimecraft_text::Localize` for enums.
#[proc_macro_derive(Localize, attributes(localize))]
pub fn derive_localize(input: TokenStream) -> TokenStream {
    derive_localize::derive_localize_impl(input)
}
