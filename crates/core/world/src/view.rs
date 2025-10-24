//! View traits.

use rimecraft_voxel_math::coord_section_from_block;

pub mod block;
pub mod chunk;
pub mod light;
mod state_option;
pub mod world;

pub use state_option::StateOption;

/// Height limit specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightLimit {
    height: u32,
    bottom: i32,
}

impl HeightLimit {
    /// Creates a new height limit.
    #[inline]
    pub const fn new(height: u32, bottom: i32) -> Self {
        Self { height, bottom }
    }

    /// Returns the difference in the [`Self::bottom`] and [`Self::top`] height.
    ///
    /// This is the number of blocks that can be modified in any vertical column
    /// within the view, or the vertical size, in blocks.
    #[inline]
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Returns the bottom Y level, inclusive.
    #[inline]
    pub const fn bottom(self) -> i32 {
        self.bottom
    }

    /// Returns the top Y level, exclusive.
    #[inline]
    pub const fn top(self) -> i32 {
        self.bottom() + self.height() as i32
    }

    /// Returns the bottom section coordinate, inclusive.
    pub const fn bottom_section_coord(self) -> i32 {
        coord_section_from_block(self.bottom())
    }

    /// Returns the top section coordinate, exclusive.
    pub const fn top_section_coord(self) -> i32 {
        coord_section_from_block(self.top() - 1) + 1
    }

    /// Returns the number of sections in the view, vertically.
    pub const fn count_vertical_sections(self) -> usize {
        (self.top_section_coord() - self.bottom_section_coord()) as usize
    }

    /// Whether the given Y level is within the view's height limit.
    pub const fn is_out_of_limit(self, y: i32) -> bool {
        y < self.bottom() || y >= self.top()
    }

    /// Returns a zero-based section index for the given Y level.
    pub const fn section_index(self, y: i32) -> usize {
        self.section_coord_to_index(coord_section_from_block(y)) as usize
    }

    /// Converts a section coordinate to a zero-based section index.
    pub const fn section_coord_to_index(self, coord: i32) -> i32 {
        coord - self.bottom_section_coord()
    }

    /// Converts a zero-based section index to a section coordinate.
    pub const fn section_index_to_coord(self, index: usize) -> i32 {
        index as i32 + self.bottom_section_coord()
    }
}
