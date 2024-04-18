//! Block views.

use std::sync::Arc;

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_block_entity::BlockEntity;
use rimecraft_fluid::{FluidState, ProvideFluidStateExtTy};
use rimecraft_voxel_math::BlockPos;

use crate::{BlockEntityCell, DEFAULT_MAX_LIGHT_LEVEL};

use super::StateOption;

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
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T;
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
    /// Peeks the [`BlockState`] at the given position.
    fn peek_block_state_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockState<'w, Cx>) -> T;

    /// Peeks the [`FluidState`] at the given position.
    fn peek_fluid_state_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s FluidState<'w, Cx>) -> T;

    /// Peeks the [`BlockEntity`] at the given position.
    fn peek_block_entity_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T;
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
    fn set_block_entity(&mut self, block_entity: Box<BlockEntity<'w, Cx>>);

    /// Removes a [`BlockEntity`] from this view, and returns it if presents.
    fn remove_block_entity(&mut self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>>;
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
    fn set_block_entity_locked(&self, block_entity: Box<BlockEntity<'w, Cx>>);

    /// Removes a [`BlockEntity`] from this view, and returns it if presents.
    fn remove_block_entity_locked(&self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>>;
}
