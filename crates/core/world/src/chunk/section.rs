use std::fmt::Debug;

use local_cx::LocalContext;
use rimecraft_block::{Block, BlockState, RawBlock};
use rimecraft_chunk_palette::{
    IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw, Maybe,
    container::{PalettedContainer, ProvidePalette},
};
use rimecraft_fluid::{BlockStateExt as _, BsToFs};
use rimecraft_registry::Registry;

use super::{ChunkCx, internal_types::*};

/// Section on a `Chunk`.
pub struct ChunkSection<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    bsc: PalettedContainer<Cx::BlockStateList, BlockState<'w, Cx>, Cx>,
    bic: PalettedContainer<Cx::BiomeList, IBiome<'w, Cx>, Cx>,

    ne_block_c: u16,
    rt_block_c: u16,
    ne_fluid_c: u16,
}

impl<'w, Cx> ChunkSection<'w, Cx>
where
    Cx: BsToFs<'w> + ChunkCx<'w>,
    Cx::BlockStateList: for<'s> PalIndexFromRaw<'s, Maybe<'s, BlockState<'w, Cx>>>,
    for<'a> &'a Cx::BlockStateList: IntoIterator,
    for<'a> <&'a Cx::BlockStateList as IntoIterator>::IntoIter: ExactSizeIterator,
{
    /// Creates a new chunk section with the given containers.
    pub fn new(
        bs_container: PalettedContainer<Cx::BlockStateList, BlockState<'w, Cx>, Cx>,
        bi_container: PalettedContainer<Cx::BiomeList, IBiome<'w, Cx>, Cx>,
    ) -> Self {
        let mut this = Self {
            bsc: bs_container,
            bic: bi_container,
            ne_block_c: 0,
            rt_block_c: 0,
            ne_fluid_c: 0,
        };
        this.calculate_counts();
        this
    }

    /// Calculate and updates the counts of non-empty blocks, random tick blocks, and non-empty fluids.
    pub fn calculate_counts(&mut self) {
        let mut ne_block_c = 0;
        let mut rt_block_c = 0;
        let mut ne_fluid_c = 0;

        self.bsc.count(|bs, count| {
            let fs = bs.to_fluid_state();
            if !bs.block.settings().is_empty {
                ne_block_c += count;
            }
            if bs.block.settings().random_ticks {
                rt_block_c += count;
            }
            if !fs.fluid.settings().is_empty {
                ne_block_c += count;
                if fs.fluid.settings().random_ticks {
                    ne_fluid_c += count;
                }
            }
        });

        self.ne_block_c = ne_block_c as u16;
        self.rt_block_c = rt_block_c as u16;
        self.ne_fluid_c = ne_fluid_c as u16;
    }
}

impl<'w, Cx> ChunkSection<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns the block state container of the chunk section.
    #[inline]
    pub fn bs_container(&self) -> &PalettedContainer<Cx::BlockStateList, BlockState<'w, Cx>, Cx> {
        &self.bsc
    }

    /// Returns the mutable block state container of the chunk section.
    #[inline]
    pub fn bs_container_mut(
        &mut self,
    ) -> &mut PalettedContainer<Cx::BlockStateList, BlockState<'w, Cx>, Cx> {
        &mut self.bsc
    }

    /// Returns the biome container of the chunk section.
    #[inline]
    pub fn bi_container(&self) -> &PalettedContainer<Cx::BiomeList, IBiome<'w, Cx>, Cx> {
        &self.bic
    }

    /// Returns the mutable biome container of the chunk section.
    #[inline]
    pub fn bi_container_mut(
        &mut self,
    ) -> &mut PalettedContainer<Cx::BiomeList, IBiome<'w, Cx>, Cx> {
        &mut self.bic
    }

    /// Whether the chunk section is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ne_block_c == 0
    }

    /// Whether the chunk sections contains random tick blocks.
    #[inline]
    pub fn has_random_tick_blocks(&self) -> bool {
        self.rt_block_c > 0
    }

    /// Whether the chunk sections contains random tick fluids.
    #[inline]
    pub fn has_random_tick_fluids(&self) -> bool {
        self.ne_fluid_c > 0
    }

    /// Whether the chunk sections receives random ticks.
    #[inline]
    pub fn has_random_ticks(&self) -> bool {
        self.has_random_tick_blocks() || self.has_random_tick_fluids()
    }
}

impl<'w, Cx> ChunkSection<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
{
    /// Returns the block state at the given position.
    ///
    /// # Panics
    ///
    /// Panics when the given position out of bounds.
    #[inline]
    pub fn block_state(&self, x: u32, y: u32, z: u32) -> BlockState<'w, Cx> {
        *self
            .bsc
            .get(Cx::compute_index(x, y, z))
            .expect("position out of bounds: {x},{y},{z}")
    }

    /// Returns the fluid state at the given position.
    ///
    /// # Panics
    ///
    /// Panics when the given position out of bounds.
    #[inline]
    pub fn fluid_state(&self, x: u32, y: u32, z: u32) -> IFluidState<'w, Cx>
    where
        Cx: BsToFs<'w>,
    {
        Cx::block_to_fluid_state(self.block_state(x, y, z))
    }
}

impl<'w, Cx> ChunkSection<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::Biome, IBiome<'w, Cx>>,
    Cx::BiomeList: for<'s> PalIndexFromRaw<'s, Maybe<'s, IBiome<'w, Cx>>>,
{
    /// Returns the biome at the given position.
    #[inline]
    pub fn biome(&self, x: u32, y: u32, z: u32) -> Option<Maybe<'_, IBiome<'w, Cx>>> {
        self.bic.get(Cx::compute_index(x, y, z))
    }
}

impl<'w, Cx> ChunkSection<'w, Cx>
where
    Cx: BsToFs<'w> + ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    Cx::BlockStateList: for<'a> PalIndexToRaw<&'a BlockState<'w, Cx>>
        + for<'s> PalIndexFromRaw<'s, Maybe<'s, BlockState<'w, Cx>>>
        + Clone,
{
    /// Sets the block state at the given position and returns the old one.
    ///
    /// # Panics
    ///
    /// Panics if the given inner position is out of bounds.
    pub fn set_block_state(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        state: BlockState<'w, Cx>,
    ) -> BlockState<'w, Cx> {
        let state_old = match self
            .bsc
            .swap(Cx::compute_index(x, y, z), state)
            .expect("position out of bounds")
        {
            Maybe::Borrowed(bs) => *bs,
            Maybe::Owned(maybe::SimpleOwned(bs)) => bs,
        };

        if !state_old.block.settings().is_empty {
            self.ne_block_c -= 1;
            if state_old.block.settings().random_ticks {
                self.rt_block_c -= 1;
            }
        }
        let fs = state_old.to_fluid_state();
        if !fs.fluid.settings().is_empty {
            self.ne_fluid_c -= 1;
        }

        if !state.block.settings().is_empty {
            self.ne_block_c += 1;
            if state.block.settings().random_ticks {
                self.rt_block_c += 1;
            }
        }
        let fs = state.to_fluid_state();
        if !fs.fluid.settings().is_empty {
            self.ne_fluid_c += 1;
        }

        state_old
    }
}

impl<'w, Cx> ChunkSection<'w, Cx>
where
    Cx: ChunkCx<'w>
        + ProvidePalette<Cx::BlockStateList, BlockState<'w, Cx>>
        + ProvidePalette<Cx::BiomeList, IBiome<'w, Cx>>,
    Cx::BlockStateList: for<'a> PalIndexToRaw<&'a BlockState<'w, Cx>>
        + for<'s> PalIndexFromRaw<'s, Maybe<'s, BlockState<'w, Cx>>>
        + Clone,
    &'w Registry<Cx::Id, Cx::Biome>: Into<Cx::BiomeList>,
    Cx::BiomeList: for<'a> PalIndexToRaw<&'a IBiome<'w, Cx>>
        + for<'s> PalIndexFromRaw<'s, Maybe<'s, IBiome<'w, Cx>>>
        + Clone,
{
    /// Creates a [`ChunkSection`] for the given `Biome` registry
    ///
    /// # Panics
    ///
    /// Panics if the biome registry doesn't contains a default entry.
    pub fn from_registries<Local>(cx: Local) -> Self
    where
        Local: LocalContext<&'w Registry<Cx::Id, Cx::Biome>>
            + LocalContext<&'w Registry<Cx::Id, RawBlock<'w, Cx>>>
            + LocalContext<Cx::BlockStateList>,
    {
        let default_block = std::convert::identity::<&Registry<_, RawBlock<'_, _>>>(cx.acquire())
            .default_entry()
            .expect("no default block found for registry");
        let registry = std::convert::identity::<&Registry<_, Cx::Biome>>(cx.acquire());

        Self {
            bsc: PalettedContainer::of_single(
                LocalContext::<Cx::BlockStateList>::acquire(cx),
                BlockState {
                    block: default_block,
                    state: Block::to_value(default_block).states().default_state(),
                },
            ),
            bic: PalettedContainer::of_single(
                registry.into(),
                registry
                    .default_entry()
                    .expect("biome registry should contains a default entry"),
            ),
            ne_block_c: 0,
            rt_block_c: 0,
            ne_fluid_c: 0,
        }
    }
}

impl<'w, Cx> Debug for ChunkSection<'w, Cx>
where
    Cx: ChunkCx<'w> + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt<'w>: Debug,
    Cx::BlockStateList: Debug,
    Cx::Biome: Debug,
    Cx::BiomeList: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkSection")
            .field("bsc", &self.bsc)
            .field("bic", &self.bic)
            .field("ne_block_c", &self.ne_block_c)
            .field("rt_block_c", &self.rt_block_c)
            .field("ne_fluid_c", &self.ne_fluid_c)
            .finish()
    }
}

/// Trait for computing the index of a position in a chunk section for [`PalettedContainer`].
pub trait ComputeIndex<L, T>: ProvidePalette<L, T> {
    /// Computes the index of the given position.
    ///
    /// The number type is unsigned because the index will overflow when it's negative.
    #[inline]
    fn compute_index(x: u32, y: u32, z: u32) -> usize {
        ((((y << Self::EDGE_BITS) | z) << Self::EDGE_BITS) | x) as usize
    }
}

#[cfg(feature = "edcode")]
mod _edcode {

    use edcode2::{Buf, BufMut, Decode, Encode};

    use super::*;

    impl<'w, Cx, B> Encode<B> for ChunkSection<'w, Cx>
    where
        Cx: ChunkCx<'w>,
        Cx::BlockStateList: for<'a> PalIndexToRaw<&'a BlockState<'w, Cx>>,
        Cx::BiomeList: for<'a> PalIndexToRaw<&'a IBiome<'w, Cx>>,
        B: BufMut,
    {
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            buf.put_i16(self.ne_block_c as i16);
            self.bsc.encode(&mut buf)?;
            self.bic.encode(&mut buf)
        }
    }

    impl<'w, 'de, Cx, B> Decode<'de, B> for ChunkSection<'w, Cx>
    where
        Cx: ChunkCx<'w>,
        Cx::BlockStateList: for<'s> PalIndexFromRaw<'s, BlockState<'w, Cx>> + Clone,
        Cx::BiomeList: for<'s> PalIndexFromRaw<'s, Maybe<'s, IBiome<'w, Cx>>>
            + for<'s> PalIndexFromRaw<'s, IBiome<'w, Cx>>
            + for<'a> PalIndexToRaw<&'a IBiome<'w, Cx>>
            + Clone,
        Cx: ProvidePalette<Cx::BlockStateList, BlockState<'w, Cx>>,
        Cx: ProvidePalette<Cx::BiomeList, IBiome<'w, Cx>>,
        B: Buf,
    {
        fn decode_in_place(&mut self, mut buf: B) -> Result<(), edcode2::BoxedError<'de>> {
            self.ne_block_c = buf.get_i16() as u16;
            self.bsc.decode_in_place(&mut buf)?;
            let mut sliced = self.bic.to_slice();
            sliced.decode_in_place(&mut buf)?;
            self.bic = sliced;
            Ok(())
        }

        #[inline]
        fn decode(_buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            panic!("chunk sections does not support non-in-place decoding")
        }

        const SUPPORT_NON_IN_PLACE: bool = false;
    }

    impl<'w, Cx> ChunkSection<'w, Cx>
    where
        Cx: ChunkCx<'w>,
        Cx::BiomeList: for<'s> PalIndexFromRaw<'s, Maybe<'s, IBiome<'w, Cx>>>
            + for<'s> PalIndexFromRaw<'s, IBiome<'w, Cx>>
            + for<'a> PalIndexToRaw<&'a IBiome<'w, Cx>>
            + Clone,
        Cx: ProvidePalette<Cx::BiomeList, IBiome<'w, Cx>>,
    {
        /// Updates the chunk section from the given biome packet buffer.
        #[allow(clippy::missing_errors_doc)]
        pub fn update_from_biome_buf<B>(
            &mut self,
            mut buf: B,
        ) -> Result<(), edcode2::BoxedError<'static>>
        where
            B: Buf,
        {
            let mut sliced = self.bic.to_slice();
            sliced.decode_in_place(&mut buf)?;
            self.bic = sliced;
            Ok(())
        }
    }
}
