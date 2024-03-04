use std::{fmt::Debug, hash::Hash, sync::Arc};

use rimecraft_block::Block;
use rimecraft_fluid::Fluid;
use rimecraft_registry::Reg;
use rimecraft_state::State;

use super::ChunkSectionTy;

/// The internal-used `Biome` type.
pub type IBiome<'w, K, Cx> = Reg<'w, K, <Cx as ChunkSectionTy<'w>>::Biome>;

/// The internal-used `BlockState` type.
///
/// This contains the block registration and the [`State`].
pub struct IBlockState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    /// The block.
    pub block: Block<'w, K, <Cx as ChunkSectionTy<'w>>::BlockStateExt, Cx>,
    /// The state.
    pub state: Arc<State<'w, <Cx as ChunkSectionTy<'w>>::BlockStateExt>>,
}

impl<'w, K, Cx> Debug for IBlockState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'w>>::BlockStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IBlockState")
            .field("block", &self.block)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, K, Cx> Clone for IBlockState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            block: self.block,
            state: self.state.clone(),
        }
    }
}

impl<'w, K, Cx> PartialEq for IBlockState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<'w, K, Cx> Eq for IBlockState<'w, K, Cx> where Cx: ChunkSectionTy<'w> {}

impl<'w, K, Cx> Hash for IBlockState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}

/// The internal-used [`IBlockState`] reference type.
pub struct IBlockStateRef<'a, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    /// The block.
    pub block: Block<'w, K, <Cx as ChunkSectionTy<'w>>::BlockStateExt, Cx>,
    /// The state.
    pub state: &'a Arc<State<'w, <Cx as ChunkSectionTy<'w>>::BlockStateExt>>,
}

impl<'w, K, Cx> Debug for IBlockStateRef<'_, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'w>>::BlockStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IBlockStateRef")
            .field("block", &self.block)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, K, Cx> Clone for IBlockStateRef<'_, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'w, K, Cx> Copy for IBlockStateRef<'_, 'w, K, Cx> where Cx: ChunkSectionTy<'w> {}

impl<'w, K, Cx> PartialEq for IBlockStateRef<'_, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && Arc::ptr_eq(self.state, other.state)
    }
}

impl<'w, K, Cx> Eq for IBlockStateRef<'_, 'w, K, Cx> where Cx: ChunkSectionTy<'w> {}

impl<'a, 'w, K, Cx> From<IBlockStateRef<'a, 'w, K, Cx>> for IBlockState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn from(value: IBlockStateRef<'a, 'w, K, Cx>) -> Self {
        Self {
            block: value.block,
            state: Arc::clone(value.state),
        }
    }
}

impl<'a, 'w, K, Cx> From<&'a IBlockState<'w, K, Cx>> for IBlockStateRef<'a, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn from(value: &'a IBlockState<'w, K, Cx>) -> Self {
        Self {
            block: value.block,
            state: &value.state,
        }
    }
}

/// The internal-used `FluidState` type.
///
/// This contains the fluid registration and the [`State`].
pub struct IFluidState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    /// The fluid.
    pub fluid: Fluid<'w, K, <Cx as ChunkSectionTy<'w>>::FluidStateExt, Cx>,
    /// The state.
    pub state: Arc<State<'w, <Cx as ChunkSectionTy<'w>>::FluidStateExt>>,
}

impl<'w, K, Cx> Debug for IFluidState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'w>>::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFluidState")
            .field("fluid", &self.fluid)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, K, Cx> Clone for IFluidState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            fluid: self.fluid,
            state: self.state.clone(),
        }
    }
}

impl<'w, K, Cx> PartialEq for IFluidState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<'w, K, Cx> Eq for IFluidState<'w, K, Cx> where Cx: ChunkSectionTy<'w> {}

impl<'w, K, Cx> Hash for IFluidState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fluid.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}

/// The internal-used [`IFluidState`] reference type.
pub struct IFluidStateRef<'a, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    /// The fluid.
    pub fluid: Fluid<'w, K, <Cx as ChunkSectionTy<'w>>::FluidStateExt, Cx>,
    /// The state.
    pub state: &'a Arc<State<'w, <Cx as ChunkSectionTy<'w>>::FluidStateExt>>,
}

impl<'w, K, Cx> Debug for IFluidStateRef<'_, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'w>>::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFluidStateRef")
            .field("fluid", &self.fluid)
            .field("state", &self.state)
            .finish()
    }
}

impl<'w, K, Cx> Clone for IFluidStateRef<'_, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'w, K, Cx> Copy for IFluidStateRef<'_, 'w, K, Cx> where Cx: ChunkSectionTy<'w> {}

impl<'w, K, Cx> PartialEq for IFluidStateRef<'_, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && Arc::ptr_eq(self.state, other.state)
    }
}

impl<'w, K, Cx> Eq for IFluidStateRef<'_, 'w, K, Cx> where Cx: ChunkSectionTy<'w> {}

impl<'a, 'w, K, Cx> From<IFluidStateRef<'a, 'w, K, Cx>> for IFluidState<'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn from(value: IFluidStateRef<'a, 'w, K, Cx>) -> Self {
        Self {
            fluid: value.fluid,
            state: Arc::clone(value.state),
        }
    }
}

impl<'a, 'w, K, Cx> From<&'a IFluidState<'w, K, Cx>> for IFluidStateRef<'a, 'w, K, Cx>
where
    Cx: ChunkSectionTy<'w>,
{
    #[inline]
    fn from(value: &'a IFluidState<'w, K, Cx>) -> Self {
        Self {
            fluid: value.fluid,
            state: &value.state,
        }
    }
}
