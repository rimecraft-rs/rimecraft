//! View traits.

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_block_entity::BlockEntity;
use rimecraft_chunk_palette::Maybe;
use rimecraft_fluid::{FluidState, ProvideFluidStateExtTy};
use rimecraft_voxel_math::{section_coord, BlockPos};

use crate::DEFAULT_MAX_LIGHT_LEVEL;

mod state_option;

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
        section_coord(self.bottom())
    }

    /// Returns the top section coordinate, exclusive.
    pub const fn top_section_coord(self) -> i32 {
        section_coord(self.top() - 1) + 1
    }

    /// Returns the number of sections in the view, vertically.
    pub const fn count_vertical_sections(self) -> i32 {
        self.top_section_coord() - self.bottom_section_coord()
    }

    /// Whether the given Y level is within the view's height limit.
    pub const fn is_out_of_limit(self, y: i32) -> bool {
        y < self.bottom() || y >= self.top()
    }

    /// Returns a zero-based section index for the given Y level.
    pub const fn section_index(self, y: i32) -> usize {
        self.section_coord_to_index(section_coord(y)) as usize
    }

    /// Converts a section coordinate to a zero-based section index.
    pub const fn section_coord_to_index(self, coord: i32) -> i32 {
        coord - self.bottom_section_coord()
    }

    /// Converts a zero-based section index to a section coordinate.
    pub const fn section_index_to_coord(self, index: i32) -> i32 {
        index + self.bottom_section_coord()
    }
}

/// A scoped, immutable view of [`BlockState`]s, [`FluidState`]s and [`BlockEntity`]s.
pub trait BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Peeks the [`BlockState`] at the given position.
    fn peek_block_state<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockState<'w, Cx>) -> T;

    /// Peeks the [`FluidState`] at the given position.
    fn peek_fluid_state<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s FluidState<'w, Cx>) -> T;

    /// Peeks the [`BlockEntity`] at the given position.
    fn peek_block_entity<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntity<'w, Cx>) -> T;
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

/// Lock-free variant of [`BlockView`].
pub trait LockFreeBlockView<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Returns the [`BlockEntity`] at the given position.
    fn block_entity_lf(&mut self, pos: BlockPos) -> Option<&BlockEntity<'w, Cx>>;

    /// Returns the [`BlockState`] at the given position.
    fn block_state_lf(&mut self, pos: BlockPos) -> StateOption<Maybe<'_, BlockState<'w, Cx>>>;

    /// Returns the [`FluidState`] at the given position.
    fn fluid_state_lf(&mut self, pos: BlockPos) -> StateOption<Maybe<'_, FluidState<'w, Cx>>>;
}

/// Mutable variant of [`BlockView`], without internal mutability.
pub trait BlockViewMut<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Sets the block state at the given position.
    ///
    /// If the target block state is changed, the old block state is returned.
    fn set_block_state(
        &mut self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        moved: bool,
    ) -> Option<BlockState<'w, Cx>>;

    /// Adds a [`BlockEntity`] to this view.
    fn set_block_entity(&mut self, block_entity: BlockEntity<'w, Cx>);
}

/// [`BlockViewMut`] with internal mutability.
pub trait LockedBlockViewMut<'w, Cx>: BlockViewMut<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Sets the block state at the given position.
    ///
    /// If the target block state is changed, the old block state is returned.
    fn set_block_state_locked(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        moved: bool,
    ) -> Option<BlockState<'w, Cx>>;

    /// Adds a [`BlockEntity`] to this view.
    fn set_block_entity_locked(&self, block_entity: BlockEntity<'w, Cx>);
}
