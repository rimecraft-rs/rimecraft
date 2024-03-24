use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

use super::ChunkCx;

/// The internal-used `Biome` type.
pub type IBiome<'w, Cx> = Reg<'w, <Cx as ProvideIdTy>::Id, <Cx as ChunkCx<'w>>::Biome>;

pub(crate) use rimecraft_block::BlockState as IBlockState;
pub(crate) use rimecraft_fluid::FluidState as IFluidState;
