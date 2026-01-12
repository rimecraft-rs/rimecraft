//! Common behaviors of block state extensions.

use rimecraft_state::State;

/// Block state extensions that could returns a luminance value from
/// the given state of the block.
#[deprecated = "this function should be provided by the nested block state extension types"]
pub trait ProvideLuminance: Sized {
    /// The luminance.
    fn luminance(&self, state: &State<'_, Self>) -> u32;
}
