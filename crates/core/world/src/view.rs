//! View traits.

use rimecraft_voxel_math::section_coord;

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
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the bottom Y level, inclusive.
    #[inline]
    pub const fn bottom(&self) -> i32 {
        self.bottom
    }

    /// Returns the top Y level, exclusive.
    #[inline]
    pub const fn top(&self) -> i32 {
        self.bottom() + self.height() as i32
    }

    /// Returns the bottom section coordinate, inclusive.
    pub const fn bottom_section_coord(&self) -> i32 {
        section_coord(self.bottom())
    }

    /// Returns the top section coordinate, exclusive.
    pub const fn top_section_coord(&self) -> i32 {
        section_coord(self.top() - 1) + 1
    }

    /// Returns the number of sections in the view, vertically.
    pub const fn count_vertical_sections(&self) -> i32 {
        self.top_section_coord() - self.bottom_section_coord()
    }

    /// Whether the given Y level is within the view's height limit.
    pub const fn out_of_limit(&self, y: i32) -> bool {
        y < self.bottom() || y >= self.top()
    }

    /// Returns a zero-based section index for the given Y level.
    pub const fn section_index(&self, y: i32) -> i32 {
        self.section_coord_to_index(section_coord(y))
    }

    /// Converts a section coordinate to a zero-based section index.
    pub const fn section_coord_to_index(&self, coord: i32) -> i32 {
        coord - self.bottom_section_coord()
    }

    /// Converts a zero-based section index to a section coordinate.
    pub const fn section_index_to_coord(&self, index: i32) -> i32 {
        index + self.bottom_section_coord()
    }
}

/// Trait for obtaining a [`HeightLimit`] from reference.
pub trait AsHeightLimit {
    /// Returns the [`HeightLimit`].
    fn as_height_limit(&self) -> HeightLimit;
}

impl<T: ?Sized> AsHeightLimit for T
where
    for<'a> &'a T: Into<HeightLimit>,
{
    #[inline]
    fn as_height_limit(&self) -> HeightLimit {
        self.into()
    }
}

impl AsHeightLimit for HeightLimit {
    #[inline]
    fn as_height_limit(&self) -> HeightLimit {
        *self
    }
}
