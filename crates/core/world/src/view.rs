//! View traits.

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_block_entity::BlockEntity;
use rimecraft_chunk_palette::Maybe;
use rimecraft_fluid::{FluidState, ProvideFluidStateExtTy};
use rimecraft_voxel_math::{section_coord, BlockPos};

use crate::DEFAULT_MAX_LIGHT_LEVEL;

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

/// A scoped, immutable view of [`BlockState`]s, [`FluidState`]s and [`BlockEntity`]s.
pub trait BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Returns the [`BlockEntity`] at the given position.
    fn block_entity(&self, pos: BlockPos) -> Option<&BlockEntity<'w, Cx>>;

    /// Returns the [`BlockState`] at the given position.
    fn block_state(&self, pos: BlockPos) -> StateOption<Maybe<'_, BlockState<'w, Cx>>>;

    /// Returns the [`FluidState`] at the given position.
    fn fluid_state(&self, pos: BlockPos) -> StateOption<Maybe<'_, FluidState<'w, Cx>>>;
}

/// View of block luminance source levels.
pub trait BlockLuminanceView<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Returns the luminance source level of the given position.
    fn luminance(&self, pos: BlockPos) -> StateOption<u32>;

    /// Returns the max light level of this view.
    ///
    /// The default one is [`DEFAULT_MAX_LIGHT_LEVEL`].
    #[inline]
    fn max_light_level(&self) -> u32 {
        DEFAULT_MAX_LIGHT_LEVEL
    }
}

/// Optional state result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum StateOption<T> {
    /// A state.
    Some(T),
    /// The void variant of the state.
    Void,
    /// No state available.
    None,
}

impl<T> StateOption<T> {
    /// Maps this optional state to another type.
    pub fn map<O, F>(self, mapper: F) -> StateOption<O>
    where
        F: FnOnce(T) -> O,
    {
        match self {
            StateOption::Some(val) => StateOption::Some(mapper(val)),
            StateOption::Void => StateOption::Void,
            StateOption::None => StateOption::None,
        }
    }
}
