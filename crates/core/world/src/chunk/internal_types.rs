use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

use super::WorldCx;

/// The internally-used `Biome` type.
pub type IBiome<'w, Cx> = Reg<'w, <Cx as ProvideIdTy>::Id, <Cx as WorldCx<'w>>::Biome>;
