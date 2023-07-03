pub mod chunk;
pub mod tick;

use crate::prelude::*;

/// A view with a height limit specification.
pub trait HeightLimitView {
    /// The difference in the [`Self::bottom_y`] and [`Self::top_y`] height.
    ///
    /// This is the number of blocks that can be modified in any vertical column
    /// within the view, or the vertical size, in blocks, of the view.
    fn height(&self) -> u32 {
        (self.top_y() - self.bottom_y()) as u32
    }

    /// The bottom Y level, or height, inclusive, of this view.
    fn bottom_y(&self) -> i32;
    /// The top Y level, or height, exclusive, of this view.
    fn top_y(&self) -> i32;

    /// The number of sections, vertically, within this view.
    fn count_vertical_sections(&self) -> i32 {
        self.top_section_coord() - self.bottom_section_coord()
    }

    /// The bottom section coord, inclusive, of this view.
    fn top_section_coord(&self) -> i32 {
        crate::util::math::ChunkSectionPos::section_coord(self.top_y() - 1) + 1
    }

    /// The top section coord, exclusive, of this view.
    fn bottom_section_coord(&self) -> i32 {
        crate::util::math::ChunkSectionPos::section_coord(self.bottom_y())
    }
}

/// Represents a scoped, read-only view of block states,
/// fluid states and block entities.
pub trait BlockView: HeightLimitView {
    /// Default max light level in Rimecraft.
    const DEF_MAX_LIGHT_LEVEL: u8 = 15;

    /// The block state at the target `pos`.
    fn block_state(&self, pos: BlockPos) -> &crate::block::BlockState;

    /// The max light level of this view.
    fn max_light_level() -> u8 {
        Self::DEF_MAX_LIGHT_LEVEL
    }
}

/// Represents a view describing lights.
pub trait LightSourceView: BlockView {
    /// Perform for each light sources in this view.
    fn for_each_sources<T: Fn(BlockPos, &crate::block::BlockState)>(&self, f: T);
}
