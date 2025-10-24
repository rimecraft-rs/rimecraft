//! Block views.
//!
//! These views all take mutable reference to the _type_ for unifying the interface of locked access
//! and lock-free access, where the latter one requires mutability.

use bitflags::bitflags;
use local_cx::ProvideLocalCxTy;
use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_block_entity::{BlockEntity, BlockEntityCell};
use rimecraft_fluid::{FluidState, ProvideFluidStateExtTy};
use rimecraft_voxel_math::BlockPos;

/// A scoped, immutable view of [`BlockState`]s and [`FluidState`]s.
pub trait BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Returns the [`BlockState`] at the given position.
    fn block_state(&self, pos: BlockPos) -> Option<BlockState<'w, Cx>>;

    /// Returns the [`FluidState`] at the given position.
    fn fluid_state(&self, pos: BlockPos) -> Option<FluidState<'w, Cx>>;
}

/// [`BlockView`] but takes mutable reference to the _type_ for unifying the interface of locked access
/// and lock-free access, where the latter one requires mutability.
pub trait MutBlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Returns the [`BlockState`] at the given position.
    fn block_state(&mut self, pos: BlockPos) -> Option<BlockState<'w, Cx>>;

    /// Returns the [`FluidState`] at the given position.
    fn fluid_state(&mut self, pos: BlockPos) -> Option<FluidState<'w, Cx>>;
}

/// A scoped, immutable view of [`BlockEntity`]s.
///
/// This is an affiliation of [`BlockView`].
pub trait BlockEntityView<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Peeks the [`BlockEntity`] at the given position.
    fn peek_block_entity<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T;
}

/// [`BlockEntityView`] but takes mutable reference to the _type_ for unifying the interface of locked access
/// and lock-free access, where the latter one requires mutability.
pub trait MutBlockEntityView<'w, Cx>: MutBlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Peeks the [`BlockEntity`] at the given position.
    fn peek_block_entity<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T;
}

bitflags! {
    /// Flags for [`BlockViewMut::set_block_state`] and friends.
    ///
    /// These flags are identical to vanilla Minecraft.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SetBlockStateFlags: u32 {
        /// Sends a neighbor update event to surrounding blocks.
        const NOTIFY_NEIGHBORS = 1u32 << 0;
        /// Notifies listeners and clients who need to react when the block changes.
        const NOTIFY_LISTENERS = 1u32 << 2;
        /// Used alongside [`NOTIFY_LISTENERS`] to suppress render pass on clients.
        const NO_REDRAW = 1u32 << 3;
        /// Forces a synchronous redraw on clients.
        const REDRAW_ON_MAIN_THREAD = 1u32 << 4;
        /// Bypass virtual block state changes and forces the passed state to be stored as-is.
        const FORCE_STATE = 1u32 << 5;
        /// Prevents the previous block or container from dropping items when destroyed.
        const SKIP_DROPS = 1u32 << 6;
        /// Signals that the current block is being moved to a different location.
        /// For example by a piston block in vanilla Minecraft.
        const MOVED = 1u32 << 7;
        /// Skips `state_for_neighbor_update` call for redstone wire in vanilla Minecraft.
        #[doc(alias = "SKIP_REDSTONE_WIRE_STATE_REPLACEMENT")]
        const SKIP_WIRE_STATE_REPLACEMENT = 1u32 << 8;
        /// Skips `BlockEntity`'s `on_block_replaced` logistics.
        const SKIP_BLOCK_ENTITY_REPLACED_CALLBACK = 1u32 << 9;
        /// Skips `Block`'s `on_block_added` logistics.
        const SKIP_BlOCK_ADDED_CALLBACK = 1u32 << 10;
    }
}

/// Mutable variant of [`BlockView`].
pub trait BlockViewMut<'w, Cx>: MutBlockView<'w, Cx>
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
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>>;
}

/// [`BlockViewMut`] with internal mutability.
pub trait ConstBlockViewMut<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Sets the block state at the given position.
    ///
    /// If the target block state is changed, the old block state is returned.
    fn set_block_state(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>>;
}

/// Mutable variant of [`BlockEntityView`], without internal mutability.
pub trait BlockEntityViewMut<'w, Cx>: MutBlockEntityView<'w, Cx> + BlockViewMut<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy + ProvideLocalCxTy,
{
    /// Adds a [`BlockEntity`] to this view.
    fn set_block_entity(&mut self, block_entity: Box<BlockEntity<'w, Cx>>);

    /// Removes a [`BlockEntity`] from this view, and returns it if presents.
    fn remove_block_entity(&mut self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>>;
}

/// [`BlockEntityViewMut`] with internal mutability.
pub trait ConstBlockEntityViewMut<'w, Cx>: ConstBlockViewMut<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy + ProvideLocalCxTy,
{
    /// Adds a [`BlockEntity`] to this view.
    fn set_block_entity(&self, block_entity: Box<BlockEntity<'w, Cx>>);

    /// Removes a [`BlockEntity`] from this view, and returns it if presents.
    fn remove_block_entity(&self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>>;
}

impl<'w, Cx, T> MutBlockView<'w, Cx> for &T
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
    T: BlockView<'w, Cx>,
{
    #[inline(always)]
    fn block_state(&mut self, pos: BlockPos) -> Option<BlockState<'w, Cx>> {
        BlockView::block_state(*self, pos)
    }

    #[inline(always)]
    fn fluid_state(&mut self, pos: BlockPos) -> Option<FluidState<'w, Cx>> {
        BlockView::fluid_state(*self, pos)
    }
}

impl<'w, Cx, T> MutBlockEntityView<'w, Cx> for &T
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
    T: BlockEntityView<'w, Cx>,
{
    #[inline(always)]
    fn peek_block_entity<F, U>(&mut self, pos: BlockPos, pk: F) -> Option<U>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> U,
    {
        BlockEntityView::peek_block_entity(*self, pos, pk)
    }
}
