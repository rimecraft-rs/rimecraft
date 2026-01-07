//! Procedural macros for deriving text-related traits.

mod derive_localize;

use proc_macro::TokenStream;

/// Derives `rimecraft_text::Localize` for enums.
///
/// Variant-level attributes:
/// - `#[localize(a, b, _, c)]` - `_` becomes variant name (snake_case)
/// - `#[localize]` - defaults to variant name
/// - `#[localize = "a.b._.c"]` - dot-separated string
///
/// Enum-level attributes:
/// - `#[localize(prefix = "...", suffix = "...")]` - `_` becomes enum name (snake_case)
/// - `#[localize(prefix = [a, _, c])]` - array syntax, `_` becomes enum name (snake_case)
///
/// Example:
/// ```
/// use rimecraft_text::Localize;
///
/// #[derive(Localize)]
/// #[localize(prefix = [game, _])]
/// enum Mode {
///     #[localize(survival)]
///     Survival,
/// }
///
/// assert_eq!(Mode::Survival.localization_key(), "game.mode.survival");
/// ```
#[proc_macro_derive(Localize, attributes(localize))]
pub fn derive_localize(input: TokenStream) -> TokenStream {
    derive_localize::derive_localize_impl(input)
}
