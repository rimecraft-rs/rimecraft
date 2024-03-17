//! Rimecraft block entity primitives.

use std::{fmt::Debug, sync::Arc};

use ahash::AHashSet;
use erased_serde::Serialize as ErasedSerialize;

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;
use rimecraft_serde_update::erased::ErasedUpdate;
use rimecraft_voxel_math::BlockPos;

/// Raw instance of [`BlockEntityType`].
#[derive(Debug)]
pub struct RawBlockEntityType {
    /// Block raw IDs this BE type targets.
    blocks: AHashSet<usize>,
}

/// A type of [`BlockEntity`].
pub type BlockEntityType<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, RawBlockEntityType>;

/// An object holding extra data about a block in a world.
pub struct BlockEntity<'a, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    ty: BlockEntityType<'a, Cx>,
    cached_state: Arc<BlockState<'a, Cx>>,

    pos: BlockPos,
    removed: bool,

    data: T,
}

/// A context representing status of a [`BlockEntity`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Context {
    /// Whether the block entity is removed.
    pub removed: bool,
    /// Position of the block entity.
    pub pos: BlockPos,
}

impl<T, Cx> Debug for BlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy + Debug,
    Cx::BlockStateExt: Debug,
    Cx::Id: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockEntity")
            .field("type", &self.ty)
            .field("pos", &self.pos)
            .field("removed", &self.removed)
            .field("cached_state", &self.cached_state)
            .field("data", &self.data)
            .finish()
    }
}

trait BEData: ErasedSerialize + for<'de> ErasedUpdate<'de> {}
