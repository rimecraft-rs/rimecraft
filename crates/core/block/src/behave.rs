//! Common behaviors of block state extensions.

use rimecraft_state::State;

/// Block state extensions that could returns a luminance value from
/// the given state of the block.
pub trait ProvideLuminance: Sized {
    /// The luminance.
    fn luminance(&self, state: &State<'_, Self>) -> u32;
}
