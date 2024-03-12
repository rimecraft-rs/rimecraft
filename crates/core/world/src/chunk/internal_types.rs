use std::{fmt::Debug, hash::Hash, sync::Arc};

use rimecraft_block::Block;
use rimecraft_fluid::Fluid;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;
use rimecraft_state::State;

use super::ChunkCx;

/// The internal-used `Biome` type.
pub type IBiome<'w, Cx> = Reg<'w, <Cx as ProvideIdTy>::Id, <Cx as ChunkCx<'w>>::Biome>;

/// The internal-used `BlockState` type.
///
/// This contains the block registration and the [`State`].
pub struct IBlockState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The block.
    pub block: Block<'w, Cx>,
    /// The state.
    pub state: Arc<State<'w, Cx::BlockStateExt>>,
}

impl<'w, Cx> Debug for IBlockState<'w, Cx>
where
    Cx: ChunkCx<'w> + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IBlockState")
            .field("block", &self.block)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, Cx> Clone for IBlockState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            block: self.block,
            state: self.state.clone(),
        }
    }
}

impl<'w, Cx> PartialEq for IBlockState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<'w, Cx> Eq for IBlockState<'w, Cx> where Cx: ChunkCx<'w> {}

impl<'w, Cx> Hash for IBlockState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}

/// The internal-used `FluidState` type.
///
/// This contains the fluid registration and the [`State`].
pub struct IFluidState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The fluid.
    pub fluid: Fluid<'w, Cx>,
    /// The state.
    pub state: Arc<State<'w, Cx::FluidStateExt>>,
}

impl<'w, Cx> Debug for IFluidState<'w, Cx>
where
    Cx: ChunkCx<'w> + Debug,
    Cx::Id: Debug,
    Cx::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFluidState")
            .field("fluid", &self.fluid)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, Cx> Clone for IFluidState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            fluid: self.fluid,
            state: self.state.clone(),
        }
    }
}

impl<'w, Cx> PartialEq for IFluidState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<'w, Cx> Eq for IFluidState<'w, Cx> where Cx: ChunkCx<'w> {}

impl<'w, Cx> Hash for IFluidState<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fluid.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}
