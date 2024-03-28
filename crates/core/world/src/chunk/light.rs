//! Chunk lighting related stuffs.

use rimecraft_packed_int_array::PackedIntArray;

use crate::view::HeightLimit;

/// Bytes stores the maximum sky light that reaches each block,
/// regardless of current time.
#[derive(Debug)]
pub struct ChunkSkyLight {
    pal: PackedIntArray,
    min_y: i32,
    // reusable block positions: unneeded in Rust
}

impl ChunkSkyLight {
    /// Creates a new chunk sky light.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(height: HeightLimit) -> Self {
        let min_y = height.bottom() - 1;
        let j = usize::BITS - ((height.top() - min_y + 1) as usize).leading_zeros();
        Self {
            pal: PackedIntArray::from_packed(j, 256, None).unwrap(),
            min_y,
        }
    }
}
