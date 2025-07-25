//! Minecraft block primitives.

use behave::ProvideLuminance;
use dsyn::{DescriptorSet, HoldDescriptors};
use rimecraft_global_cx::{GlobalContext, ProvideIdTy};
use rimecraft_registry::Reg;
use rimecraft_state::{State, States};

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub mod behave;

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
    descriptors: DescriptorSet<'static, 'a>,
}

impl<'a, Cx> RawBlock<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Creates a new block with the given settings.
    #[inline]
    pub const fn new(
        settings: Settings,
        states: States<'a, Cx::BlockStateExt>,
        descriptors: DescriptorSet<'static, 'a>,
    ) -> Self {
        Self {
            settings,
            states,
            _marker: PhantomData,
            descriptors,
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

impl<'a, Cx> HoldDescriptors<'static, 'a> for RawBlock<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    #[inline]
    fn descriptors(&self) -> &DescriptorSet<'static, 'a> {
        &self.descriptors
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
pub trait ProvideBlockStateExtTy: ProvideIdTy {
    /// The type of the block state extension.
    type BlockStateExt;
}

/// The `BlockState` type.
///
/// This contains the block registration and the [`State`].
pub struct BlockState<'w, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// The block.
    pub block: Block<'w, Cx>,

    /// The state.
    pub state: &'w State<'w, Cx::BlockStateExt>,
}

impl<Cx> BlockState<'_, Cx>
where
    Cx: ProvideBlockStateExtTy,
    Cx::BlockStateExt: ProvideLuminance,
{
    /// Returns the luminance level of this block state.
    #[inline]
    pub fn luminance(&self) -> u32 {
        self.state.data().luminance(self.state)
    }
}

impl<Cx> Debug for BlockState<'_, Cx>
where
    Cx: ProvideBlockStateExtTy + Debug,
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

impl<Cx> Copy for BlockState<'_, Cx> where Cx: ProvideBlockStateExtTy {}

impl<Cx> Clone for BlockState<'_, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Cx> PartialEq for BlockState<'_, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && std::ptr::eq(self.state, other.state)
    }
}

impl<Cx> Eq for BlockState<'_, Cx> where Cx: ProvideBlockStateExtTy {}

impl<Cx> Hash for BlockState<'_, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.hash(state);
        std::ptr::from_ref(self.state).hash(state);
    }
}
