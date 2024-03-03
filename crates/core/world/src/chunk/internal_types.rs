use std::{fmt::Debug, hash::Hash, sync::Arc};

use rimecraft_block::Block;
use rimecraft_fluid::Fluid;
use rimecraft_registry::Reg;
use rimecraft_state::State;

use super::ChunkSectionTy;

/// The internal-used `Biome` type.
pub type IBiome<'bs, 'bi, K, Cx> = Reg<'bi, K, <Cx as ChunkSectionTy<'bs, 'bi>>::Biome>;

/// The internal-used `BlockState` type.
///
/// This contains the block registration and the [`State`].
pub struct IBlockState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    /// The block.
    pub block: Block<'bs, K, <Cx as ChunkSectionTy<'bs, 'bi>>::BlockStateExt, Cx>,
    /// The state.
    pub state: Arc<State<'bs, <Cx as ChunkSectionTy<'bs, 'bi>>::BlockStateExt>>,
}

impl<'bs, 'bi, K, Cx> Debug for IBlockState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'bs, 'bi>>::BlockStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IBlockState")
            .field("block", &self.block)
            .field("state", &self.state)
            .finish()
    }
}

impl<'bs, 'bi, K, Cx> Clone for IBlockState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            block: self.block,
            state: self.state.clone(),
        }
    }
}

impl<'bs, 'bi, K, Cx> PartialEq for IBlockState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<'bs, 'bi, K, Cx> Eq for IBlockState<'bs, 'bi, K, Cx> where Cx: ChunkSectionTy<'bs, 'bi> {}

impl<'bs, 'bi, K, Cx> Hash for IBlockState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}

/// The internal-used [`IBlockState`] reference type.
pub struct IBlockStateRef<'a, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    /// The block.
    pub block: Block<'bs, K, <Cx as ChunkSectionTy<'bs, 'bi>>::BlockStateExt, Cx>,
    /// The state.
    pub state: &'a Arc<State<'bs, <Cx as ChunkSectionTy<'bs, 'bi>>::BlockStateExt>>,
}

impl<'bs, 'bi, K, Cx> Debug for IBlockStateRef<'_, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'bs, 'bi>>::BlockStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IBlockStateRef")
            .field("block", &self.block)
            .field("state", &self.state)
            .finish()
    }
}

impl<'bs, 'bi, K, Cx> Clone for IBlockStateRef<'_, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'bs, 'bi, K, Cx> Copy for IBlockStateRef<'_, 'bs, 'bi, K, Cx> where Cx: ChunkSectionTy<'bs, 'bi>
{}

impl<'bs, 'bi, K, Cx> PartialEq for IBlockStateRef<'_, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && Arc::ptr_eq(self.state, other.state)
    }
}

impl<'bs, 'bi, K, Cx> Eq for IBlockStateRef<'_, 'bs, 'bi, K, Cx> where Cx: ChunkSectionTy<'bs, 'bi> {}

impl<'a, 'bs, 'bi, K, Cx> From<IBlockStateRef<'a, 'bs, 'bi, K, Cx>> for IBlockState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn from(value: IBlockStateRef<'a, 'bs, 'bi, K, Cx>) -> Self {
        Self {
            block: value.block,
            state: Arc::clone(value.state),
        }
    }
}

impl<'a, 'bs, 'bi, K, Cx> From<&'a IBlockState<'bs, 'bi, K, Cx>>
    for IBlockStateRef<'a, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn from(value: &'a IBlockState<'bs, 'bi, K, Cx>) -> Self {
        Self {
            block: value.block,
            state: &value.state,
        }
    }
}

/// The internal-used `FluidState` type.
///
/// This contains the fluid registration and the [`State`].
pub struct IFluidState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    /// The fluid.
    pub fluid: Fluid<'bs, K, <Cx as ChunkSectionTy<'bs, 'bi>>::FluidStateExt, Cx>,
    /// The state.
    pub state: Arc<State<'bs, <Cx as ChunkSectionTy<'bs, 'bi>>::FluidStateExt>>,
}

impl<'bs, 'bi, K, Cx> Debug for IFluidState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'bs, 'bi>>::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFluidState")
            .field("fluid", &self.fluid)
            .field("state", &self.state)
            .finish()
    }
}

impl<'bs, 'bi, K, Cx> Clone for IFluidState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            fluid: self.fluid,
            state: self.state.clone(),
        }
    }
}

impl<'bs, 'bi, K, Cx> PartialEq for IFluidState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<'bs, 'bi, K, Cx> Eq for IFluidState<'bs, 'bi, K, Cx> where Cx: ChunkSectionTy<'bs, 'bi> {}

impl<'bs, 'bi, K, Cx> Hash for IFluidState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fluid.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}

/// The internal-used [`IFluidState`] reference type.
pub struct IFluidStateRef<'a, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    /// The fluid.
    pub fluid: Fluid<'bs, K, <Cx as ChunkSectionTy<'bs, 'bi>>::FluidStateExt, Cx>,
    /// The state.
    pub state: &'a Arc<State<'bs, <Cx as ChunkSectionTy<'bs, 'bi>>::FluidStateExt>>,
}

impl<'bs, 'bi, K, Cx> Debug for IFluidStateRef<'_, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + Debug,
    K: Debug,
    <Cx as ChunkSectionTy<'bs, 'bi>>::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFluidStateRef")
            .field("fluid", &self.fluid)
            .field("state", &self.state)
            .finish()
    }
}

impl<'bs, 'bi, K, Cx> Clone for IFluidStateRef<'_, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'bs, 'bi, K, Cx> Copy for IFluidStateRef<'_, 'bs, 'bi, K, Cx> where Cx: ChunkSectionTy<'bs, 'bi>
{}

impl<'bs, 'bi, K, Cx> PartialEq for IFluidStateRef<'_, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && Arc::ptr_eq(self.state, other.state)
    }
}

impl<'bs, 'bi, K, Cx> Eq for IFluidStateRef<'_, 'bs, 'bi, K, Cx> where Cx: ChunkSectionTy<'bs, 'bi> {}

impl<'a, 'bs, 'bi, K, Cx> From<IFluidStateRef<'a, 'bs, 'bi, K, Cx>> for IFluidState<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn from(value: IFluidStateRef<'a, 'bs, 'bi, K, Cx>) -> Self {
        Self {
            fluid: value.fluid,
            state: Arc::clone(value.state),
        }
    }
}

impl<'a, 'bs, 'bi, K, Cx> From<&'a IFluidState<'bs, 'bi, K, Cx>>
    for IFluidStateRef<'a, 'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    #[inline]
    fn from(value: &'a IFluidState<'bs, 'bi, K, Cx>) -> Self {
        Self {
            fluid: value.fluid,
            state: &value.state,
        }
    }
}
