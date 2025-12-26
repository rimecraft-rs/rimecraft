//! Minecraft block primitives.

use dsyn::{DescriptorSet, HoldDescriptors};
use rimecraft_global_cx::{GlobalContext, ProvideIdTy};
use rimecraft_registry::Reg;
use rimecraft_state::{State, States};

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub mod behave;

mod hit;

pub use hit::*;
pub use rimecraft_state as state;

/// Block containing settings and the state manager.
#[derive(Debug)]
pub struct RawBlock<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    settings: Settings<'a, Cx>,
    states: States<'a, Cx::BlockStateExt<'a>>,
    _marker: PhantomData<Cx>,
    descriptors: DescriptorSet<'static, 'a>,
}

impl<'a, Cx> RawBlock<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Creates a new block with the given settings.
    ///
    /// # Panics
    ///
    /// Panics when the settings are invalid in the following case:
    ///
    /// - `opaque` and `transparent` are both `true`.
    pub const fn new(
        settings: Settings<'a, Cx>,
        states: States<'a, Cx::BlockStateExt<'a>>,
        descriptors: DescriptorSet<'static, 'a>,
    ) -> Self {
        assert!(
            !(settings.opaque && settings.transparent),
            "block cannot be both opaque and transparent"
        );

        Self {
            settings,
            states,
            _marker: PhantomData,
            descriptors,
        }
    }

    /// Returns the settings of the block.
    #[inline]
    pub fn settings(&self) -> &Settings<'a, Cx> {
        &self.settings
    }

    /// Returns the state manager of the block.
    #[inline]
    pub fn states(&self) -> &States<'a, Cx::BlockStateExt<'a>> {
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

/// The maximum opacity of a block.
pub const MAX_OPACITY: u8 = 15;
/// The commonly-found opacity of semi-transparent blocks.
pub const SEMI_TRANSPARENT_OPACITY: u8 = 1;

/// Settings of a block.
#[derive(Debug)]
#[non_exhaustive]
pub struct Settings<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
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
    pub empty: bool,
    /// Whether this block is opaque.
    pub opaque: bool,
    /// Whether this block is transparent.
    pub transparent: bool,

    /// Whether the block's transparency depends on the side of the block.
    ///
    /// By default this simply returns `false`.
    pub has_sided_transparency: fn(&BlockState<'a, Cx>) -> bool,

    /// The opacity of this block from zero to [`MAX_OPACITY`].
    ///
    /// By default this returns `MAX_OPACITY` if the block is opaque (see [`BlockStateExt::is_opaque_full_cube`]),
    /// zero if the block is transparent (see [`Self::transparent`])  and [`SEMI_TRANSPARENT_OPACITY`] otherwise.
    pub opacity: fn(&BlockState<'a, Cx>) -> u8,
}

impl<Cx> Default for Settings<'_, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    fn default() -> Self {
        Self {
            collidable: true,
            resistance: 0.0,
            hardness: 0.0,
            random_ticks: false,
            empty: false,
            opaque: true,
            transparent: false,

            has_sided_transparency: |_| false,
            opacity: |state| {
                if state.data().is_opaque_full_cube() {
                    MAX_OPACITY
                } else if state.settings().transparent {
                    0
                } else {
                    SEMI_TRANSPARENT_OPACITY
                }
            },
        }
    }
}

#[doc(alias = "BlockProperties")]
pub use Settings as BlockSettings;

/// Global contexts providing global `BlockState` IDs.
///
/// # MCJE Reference
///
/// This is the equivalent of `net.minecraft.block.Block.STATE_IDS` in MCJE.
#[deprecated = "this should be provided by local contexts"]
pub trait ProvideStateIds: GlobalContext {
    /// The type of the state IDs.
    type List;

    /// Returns the state IDs.
    fn state_ids() -> Self::List;
}

/// Global contexts providing block state extensions.
pub trait ProvideBlockStateExtTy: ProvideIdTy {
    /// The type of the block state extension.
    type BlockStateExt<'a>: BlockStateExt<'a, Self>;
}

/// Fundamental functions a block entity extension type should provide.
pub trait BlockStateExt<'a, Cx> {
    /// Whether this block state is full cube and its opaque.
    fn is_opaque_full_cube(&self) -> bool;

    /// The opacity of this block.
    fn opacity(&self) -> u8;
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
    pub state: &'w State<'w, Cx::BlockStateExt<'w>>,
}

impl<'w, Cx> BlockState<'w, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Whether the block's transparency depends on the side of the block.
    #[inline]
    pub fn has_sided_transparency(&self) -> bool {
        (self.block.settings.has_sided_transparency)(self)
    }

    /// Extension data of this block entity.
    #[inline]
    pub fn data(&self) -> &'w Cx::BlockStateExt<'w> {
        self.state.data()
    }

    /// Returns the settings of this block.
    #[inline]
    pub fn settings(&self) -> &Settings<'w, Cx> {
        self.block.settings()
    }
}

#[allow(deprecated)]
impl<'a, Cx> BlockState<'a, Cx>
where
    Cx: ProvideBlockStateExtTy,
    Cx::BlockStateExt<'a>: behave::ProvideLuminance,
{
    /// Returns the luminance level of this block state.
    #[inline]
    #[deprecated = "this function should be provided by the nested block state extension types"]
    pub fn luminance(&self) -> u32 {
        use behave::ProvideLuminance as _;
        self.state.data().luminance(self.state)
    }
}

impl<'a, Cx> Debug for BlockState<'a, Cx>
where
    Cx: ProvideBlockStateExtTy + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt<'a>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockState")
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
