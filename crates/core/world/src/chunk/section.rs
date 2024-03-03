use std::fmt::Debug;

use rimecraft_chunk_palette::{
    container::PalettedContainer, IndexFromRaw as PalIndexFromRaw, IndexToRaw as PalIndexToRaw,
};

use super::*;

/// Section on a `Chunk`.
pub struct ChunkSection<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    bsc: PalettedContainer<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>, Cx>,
    bic: PalettedContainer<Cx::BiomeList, IBiome<'bs, 'bi, K, Cx>, Cx>,

    ne_block_c: u16,
    rt_block_c: u16,
    ne_fluid_c: u16,
}

impl<'bs, 'bi, K, Cx> ChunkSection<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
    Cx::BlockStateList: for<'s> PalIndexFromRaw<'s, &'s IBlockState<'bs, 'bi, K, Cx>>,

    for<'a> &'a Cx::BlockStateList: IntoIterator,
    for<'a> <&'a Cx::BlockStateList as IntoIterator>::IntoIter: ExactSizeIterator,

    for<'a> &'a Cx::BlockStateExt: Into<IFluidStateRef<'a, 'bs, 'bi, K, Cx>>,
{
    /// Creates a new chunk section with the given containers.
    #[inline]
    pub fn new(
        bs_container: PalettedContainer<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>, Cx>,
        bi_container: PalettedContainer<Cx::BiomeList, IBiome<'bs, 'bi, K, Cx>, Cx>,
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

        self.bsc.count(|IBlockState { block, state }, count| {
            let fs: IFluidStateRef<'_, '_, '_, _, _> = state.data().into();
            if !block.settings().is_empty {
                ne_block_c += count;
            }
            if block.settings().random_ticks {
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

impl<'bs, 'bi, K, Cx> ChunkSection<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi>,
{
    /// Returns the block state container of the chunk section.
    #[inline]
    pub fn bs_container(
        &self,
    ) -> &PalettedContainer<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>, Cx> {
        &self.bsc
    }

    /// Returns the mutable block state container of the chunk section.
    #[inline]
    pub fn bs_container_mut(
        &mut self,
    ) -> &mut PalettedContainer<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>, Cx> {
        &mut self.bsc
    }

    /// Returns the biome container of the chunk section.
    #[inline]
    pub fn bi_container(&self) -> &PalettedContainer<Cx::BiomeList, IBiome<'bs, 'bi, K, Cx>, Cx> {
        &self.bic
    }

    /// Returns the mutable biome container of the chunk section.
    #[inline]
    pub fn bi_container_mut(
        &mut self,
    ) -> &mut PalettedContainer<Cx::BiomeList, IBiome<'bs, 'bi, K, Cx>, Cx> {
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

impl<'bs, 'bi, K, Cx> ChunkSection<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + ComputeIndex<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>>,
    Cx::BlockStateList: for<'s> PalIndexFromRaw<'s, &'s IBlockState<'bs, 'bi, K, Cx>>,
{
    /// Returns the block state at the given position.
    #[inline]
    pub fn block_state(
        &self,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<IBlockStateRef<'_, 'bs, 'bi, K, Cx>> {
        self.bsc.get(Cx::compute_index(x, y, z)).map(From::from)
    }

    /// Returns the fluid state at the given position.
    #[inline]
    pub fn fluid_state(&self, x: u32, y: u32, z: u32) -> Option<IFluidStateRef<'_, 'bs, 'bi, K, Cx>>
    where
        for<'a> &'a Cx::BlockStateExt: Into<IFluidStateRef<'a, 'bs, 'bi, K, Cx>>,
    {
        self.block_state(x, y, z)
            .map(|IBlockStateRef { state, .. }| state.data().into())
    }
}

impl<'bs, 'bi, K, Cx> ChunkSection<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + ComputeIndex<Cx::Biome, IBiome<'bs, 'bi, K, Cx>>,
    Cx::BiomeList: for<'s> PalIndexFromRaw<'s, &'s IBiome<'bs, 'bi, K, Cx>>,
{
    /// Returns the biome at the given position.
    #[inline]
    pub fn biome(&self, x: u32, y: u32, z: u32) -> Option<IBiome<'bs, 'bi, K, Cx>> {
        self.bic.get(Cx::compute_index(x, y, z)).copied()
    }
}

impl<'bs, 'bi, K, Cx> ChunkSection<'bs, 'bi, K, Cx>
where
    Cx: ChunkSectionTy<'bs, 'bi> + ComputeIndex<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>>,
    Cx::BlockStateList: for<'a> PalIndexToRaw<&'a IBlockState<'bs, 'bi, K, Cx>>
        + for<'s> PalIndexFromRaw<'s, &'s IBlockState<'bs, 'bi, K, Cx>>
        + Clone,
    for<'a> &'a Cx::BlockStateExt: Into<IFluidStateRef<'a, 'bs, 'bi, K, Cx>>,
{
    /// Sets the block state at the given position and returns the
    /// old one if present.
    #[inline]
    pub fn set_block_state(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        state: IBlockState<'bs, 'bi, K, Cx>,
    ) -> Option<IBlockStateRef<'_, 'bs, 'bi, K, Cx>> {
        let bs_old = self.bsc.swap(Cx::compute_index(x, y, z), state.clone());

        if let Some(state_old) = bs_old {
            if !state_old.block.settings().is_empty {
                self.ne_block_c -= 1;
                if state_old.block.settings().random_ticks {
                    self.rt_block_c -= 1;
                }
            }
            let fs: IFluidStateRef<'_, '_, '_, _, _> = state_old.state.data().into();
            if !fs.fluid.settings().is_empty {
                self.ne_fluid_c -= 1;
            }

            if !state.block.settings().is_empty {
                self.ne_block_c += 1;
                if state.block.settings().random_ticks {
                    self.rt_block_c += 1;
                }
            }

            let fs: IFluidStateRef<'_, '_, '_, _, _> = state.state.data().into();
            if !fs.fluid.settings().is_empty {
                self.ne_fluid_c += 1;
            }
        }

        bs_old.map(From::from)
    }
}

impl<'bs, 'bi, K, Cx> Debug for ChunkSection<'bs, 'bi, K, Cx>
where
    K: Debug,
    Cx: ChunkSectionTy<'bs, 'bi> + Debug,
    Cx::BlockStateExt: Debug,
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

#[cfg(feature = "edcode")]
mod _edcode {
    use rimecraft_edcode::{Encode, Update};

    use super::*;

    impl<'bs, 'bi, K, Cx> Encode for ChunkSection<'bs, 'bi, K, Cx>
    where
        Cx: ChunkSectionTy<'bs, 'bi>,
        Cx::BlockStateList: for<'a> PalIndexToRaw<&'a IBlockState<'bs, 'bi, K, Cx>>,
        Cx::BiomeList: for<'a> PalIndexToRaw<&'a IBiome<'bs, 'bi, K, Cx>>,
    {
        fn encode<B>(&self, mut buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            buf.put_i16(self.ne_block_c as i16);
            self.bsc.encode(&mut buf)?;
            self.bic.encode(&mut buf)
        }
    }

    impl<'bs, 'bi, K, Cx> Update for ChunkSection<'bs, 'bi, K, Cx>
    where
        Cx: ChunkSectionTy<'bs, 'bi>,

        Cx::BlockStateList: for<'s> PalIndexFromRaw<'s, IBlockState<'bs, 'bi, K, Cx>> + Clone,
        Cx::BiomeList: for<'s> PalIndexFromRaw<'s, &'s IBiome<'bs, 'bi, K, Cx>>
            + for<'s> PalIndexFromRaw<'s, IBiome<'bs, 'bi, K, Cx>>
            + for<'a> PalIndexToRaw<&'a IBiome<'bs, 'bi, K, Cx>>
            + Clone,

        Cx: ProvidePalette<Cx::BlockStateList, IBlockState<'bs, 'bi, K, Cx>>,
        Cx: ProvidePalette<Cx::BiomeList, IBiome<'bs, 'bi, K, Cx>>,
    {
        fn update<B>(&mut self, mut buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            self.ne_block_c = buf.get_i16() as u16;
            self.bsc.update(&mut buf)?;
            let mut sliced = self.bic.to_slice();
            sliced.update(&mut buf)?;
            self.bic = sliced;
            Ok(())
        }
    }
}
