//! Minecraft block primitives.

use rimecraft_global_cx::{GlobalContext, ProvideIdTy};
use rimecraft_registry::{ProvideRegistry, Reg};
use rimecraft_state::{States, StatesMut};

use std::marker::PhantomData;

pub use rimecraft_state as state;

/// Block containing settings and the state manager.
#[derive(Debug)]
pub struct RawBlock<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    settings: Settings,
    states: States<'a, Cx::BlockStateExt>,
    _marker: PhantomData<Cx>,
}

impl<'a, Cx> RawBlock<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Creates a new block with the given settings.
    #[inline]
    pub const fn new(settings: Settings, states: States<'a, Cx::BlockStateExt>) -> Self {
        Self {
            settings,
            states,
            _marker: PhantomData,
        }
    }

    /// Returns the settings of the block.
    #[inline]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Returns the state manager of the block.
    #[inline]
    pub fn states(&self) -> &States<'a, Cx::BlockStateExt> {
        &self.states
    }
}

impl<Cx> From<Settings> for RawBlock<'_, Cx>
where
    Cx: ProvideBlockStateExtTy,
    Cx::BlockStateExt: Default + Clone,
{
    fn from(settings: Settings) -> Self {
        Self::new(settings, StatesMut::new(Default::default()).freeze())
    }
}

impl<'r, K, Cx> ProvideRegistry<'r, K, Self> for RawBlock<'r, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideRegistry<'r, K, Self>,
{
    #[inline(always)]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
    }
}

/// A voxel in a `World`.
pub type Block<'a, Cx> = Reg<'a, <Cx as ProvideIdTy>::Id, RawBlock<'a, Cx>>;

/// Settings of a block.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
    /// Whether this block can be collided with.
    pub collidable: bool,
    /// The resistance of this block.
    pub resistance: f32,
    /// The hardness of this block.
    pub hardness: f32,
    /// Whether this block accepts random ticks.
    pub random_ticks: bool,
    /// Whether this block is empty.
    #[doc(alias = "is_air")]
    pub is_empty: bool,
    /// Whether this block is opaque.
    pub opaque: bool,
}

#[doc(alias = "BlockProperties")]
pub use Settings as BlockSettings;

/// Global contexts providing global `BlockState` IDs.
///
/// # MCJE Reference
///
/// This is the equivalent of `net.minecraft.block.Block.STATE_IDS` in MCJE.
pub trait ProvideStateIds: GlobalContext {
    /// The type of the state IDs.
    type List;

    /// Returns the state IDs.
    fn state_ids() -> Self::List;
}

/// Global contexts providing block state extensions.
pub trait ProvideBlockStateExtTy: GlobalContext {
    /// The type of the block state extension.
    type BlockStateExt;
}
