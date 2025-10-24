//! Iterator implementations for chunk.

use std::{fmt::Debug, iter::FusedIterator, marker::PhantomData, ops::Deref};

use glam::UVec3;
use rimecraft_block::BlockState;
use rimecraft_voxel_math::{BlockPos, ChunkSectionPos};

use crate::chunk::{
    BORDER_LEN, BaseChunkAccess, WorldCx, ChunkSection, SECTION_HEIGHT, section::ComputeIndex,
};

/// An iterator over all blocks in a chunk.
///
/// This is returned by [`super::Chunk::blocks`].
#[repr(transparent)] // this is important as we need to do some hacks
pub struct Blocks<'w, I, S, Cx>(BlocksInner<'w, I, S, Cx>)
where
    Cx: WorldCx<'w>,
    I: Iterator<Item = (ChunkSectionPos, S)>;

pub(super) fn blocks<'w, Cx, Access>(
    chunk: Access,
) -> Blocks<
    'w,
    impl DoubleEndedIterator<Item = (ChunkSectionPos, Access::ChunkSectionRead)>,
    Access::ChunkSectionRead,
    Cx,
>
where
    Cx: WorldCx<'w>,
    Access: BaseChunkAccess<'w, Cx>,
{
    let hl: crate::view::HeightLimit = chunk.bca_as_bc().height_limit;
    let mut sections_iter = chunk.iter_read_chunk_sections();
    let (pos, section) = sections_iter
        .next()
        .expect("at least one section have to exist in the chunk");

    Blocks(BlocksInner {
        upcoming_filtered: None,
        x: 0,
        y: 0,
        z: 0,
        section_block_pos: pos.min_pos(),
        remaining_sections: hl.count_vertical_sections(),
        sections_iter,
        section,
        _marker: PhantomData,
    })
}

impl<'w, I, S, Cx> FusedIterator for Blocks<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    I: DoubleEndedIterator<Item = (ChunkSectionPos, S)>,
    S: Deref<Target = ChunkSection<'w, Cx>>,
{
}

impl<'w, I, S, Cx> ExactSizeIterator for Blocks<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    I: DoubleEndedIterator<Item = (ChunkSectionPos, S)>,
    S: Deref<Target = ChunkSection<'w, Cx>>,
{
    fn len(&self) -> usize {
        let all =
            (self.0.remaining_sections + 1) as u32 * (BORDER_LEN * BORDER_LEN * SECTION_HEIGHT);
        let iterated = self.0.x + self.0.y * BORDER_LEN + self.0.z * BORDER_LEN * BORDER_LEN;
        (all - iterated) as usize
    }
}

impl<'w, I, S, Cx> Iterator for Blocks<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    I: DoubleEndedIterator<Item = (ChunkSectionPos, S)>,
    S: Deref<Target = ChunkSection<'w, Cx>>,
{
    type Item = (BlockPos, BlockState<'w, Cx>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.0.__find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.0.__find_map(f)
    }

    fn filter<P>(mut self, mut predicate: P) -> std::iter::Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.0.make_filtered(|t| predicate(&t));
        let filter_inner: std::iter::Filter<BlocksInner<'w, I, S, Cx>, P> =
            self.0.filter(predicate);
        //SAFETY: transparent representation guarantees this to be safe in low-level
        let filter: std::iter::Filter<Blocks<'w, I, S, Cx>, P> =
            unsafe { rcutil::transmute(filter_inner) };
        filter
    }

    fn filter_map<B, F>(mut self, mut f: F) -> std::iter::FilterMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.0.make_filtered(|t| f(t).is_some());
        let filter_map_inner: std::iter::FilterMap<BlocksInner<'w, I, S, Cx>, F> =
            self.0.filter_map(f);
        //SAFETY: transparent representation guarantees this to be safe in low-level
        let filter_map: std::iter::FilterMap<Blocks<'w, I, S, Cx>, F> =
            unsafe { rcutil::transmute(filter_map_inner) };
        filter_map
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

struct BlocksInner<'w, I, S, Cx>
where
    Cx: WorldCx<'w>,
    I: Iterator<Item = (ChunkSectionPos, S)>,
{
    sections_iter: I,
    section: S,

    upcoming_filtered: Option<Vec<(ChunkSectionPos, S)>>,

    // update ordering: x -> z -> y
    x: u32,
    y: u32,
    z: u32,

    section_block_pos: BlockPos,
    remaining_sections: usize, // inclusive

    _marker: PhantomData<&'w Cx>,
}

impl<'w, I, S, Cx> BlocksInner<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    I: DoubleEndedIterator<Item = (ChunkSectionPos, S)>,
    S: Deref<Target = ChunkSection<'w, Cx>>,
{
    fn make_filtered<P>(&mut self, mut predicate: P)
    where
        P: FnMut(<Self as Iterator>::Item) -> bool,
    {
        let old = self.upcoming_filtered.as_deref();
        let mut collected = self
            .sections_iter
            .by_ref()
            .rev()
            .filter(|(pos, s)| {
                let min_bp = pos.min_pos();
                s.bs_container()
                    .iter_palette()
                    .any(|bs| predicate((min_bp, *bs)))
                    && old.is_none_or(|o| o.iter().any(|(p, _)| p == pos))
            })
            .collect::<Vec<_>>();

        self.remaining_sections = collected.len();
        if let Some((cp, s)) = collected.pop() {
            self.section = s;
            self.section_block_pos = cp.min_pos();
        }
        self.upcoming_filtered = Some(collected);
    }

    fn forward_section_cursor<P>(&mut self, mut predicate: P) -> bool
    where
        P: FnMut(<Self as Iterator>::Item) -> bool,
    {
        (|| -> Option<()> {
            if let Some(cached) = self.upcoming_filtered.as_mut() {
                loop {
                    let (cp, s) = cached.pop()?;
                    self.remaining_sections -= 1;
                    let min_bp = cp.min_pos();
                    if s.bs_container()
                        .iter_palette()
                        .map(|bs| (min_bp, *bs))
                        .any(&mut predicate)
                    {
                        break;
                    }
                }
            } else {
                self.sections_iter
                    .by_ref()
                    .inspect(|_| self.remaining_sections -= 1)
                    .find(|(cp, s)| {
                        let min_bp = cp.min_pos();
                        s.bs_container()
                            .iter_palette()
                            .map(|bs| (min_bp, *bs))
                            .any(&mut predicate)
                    })?;
            }
            None
        })()
        .is_some()
    }

    #[inline]
    fn __find<P>(&mut self, mut predicate: P) -> Option<<Self as Iterator>::Item>
    where
        Self: Sized,
        P: FnMut(&<Self as Iterator>::Item) -> bool,
    {
        // the lazy operator makes this way more efficient when the iterator is already filtered
        // as the `next` implementation in `Filter` is by `find` for now
        if self.upcoming_filtered.is_none() && !self.forward_section_cursor(|t| predicate(&t)) {
            return None;
        }

        self.find(predicate)
    }

    #[inline]
    fn __find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(<Self as Iterator>::Item) -> Option<B>,
    {
        // same reason as above. `FilterMap` uses `find_map` for implementation and we have to give this up
        if self.upcoming_filtered.is_none() && !self.forward_section_cursor(|t| f(t).is_some()) {
            return None;
        }

        self.find_map(f)
    }
}

impl<'w, I, S, Cx> Iterator for BlocksInner<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
    I: Iterator<Item = (ChunkSectionPos, S)>,
    S: Deref<Target = ChunkSection<'w, Cx>>,
{
    type Item = (BlockPos, BlockState<'w, Cx>);

    fn next(&mut self) -> Option<Self::Item> {
        // or this cant be a fused iterator
        if self.remaining_sections == 0 {
            return None;
        }

        if self.x >= BORDER_LEN {
            self.x = 0;
            self.z += 1;
        }

        if self.z >= BORDER_LEN {
            self.z = 0;
            self.y += 1;
        }

        if self.y >= SECTION_HEIGHT {
            self.y = 0;

            // reset all
            (self.x, self.z) = (0, 0);

            let (section_pos, next_section) = self
                .upcoming_filtered
                .as_mut()
                .map_or_else(|| self.sections_iter.by_ref().next(), |cached| cached.pop())?;
            self.remaining_sections -= 1;
            self.section_block_pos = section_pos.min_pos();
            self.section = next_section;
        }

        self.x += 1;

        // this includes air as well
        Some((
            self.section_block_pos + UVec3::new(self.x, self.y, self.z).as_ivec3(),
            self.section.block_state(self.x, self.y, self.z),
        ))
    }
}

impl<'w, I, S, Cx> Debug for BlocksInner<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + Debug,
    I: Iterator<Item = (ChunkSectionPos, S)> + Debug,
    S: Deref<Target = ChunkSection<'w, Cx>>,
    Cx::Id: Debug,
    Cx::BlockStateExt<'w>: Debug,
    Cx::BlockStateList: Debug,
    Cx::Biome: Debug,
    Cx::BiomeList: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blocks")
            .field("sections_iter", &self.sections_iter)
            .field("section", &*self.section)
            .finish_non_exhaustive()
    }
}

impl<'w, I, S, Cx> Debug for Blocks<'w, I, S, Cx>
where
    Cx: WorldCx<'w> + Debug,
    I: Iterator<Item = (ChunkSectionPos, S)> + Debug,
    S: Deref<Target = ChunkSection<'w, Cx>>,
    Cx::Id: Debug,
    Cx::BlockStateExt<'w>: Debug,
    Cx::BlockStateList: Debug,
    Cx::Biome: Debug,
    Cx::BiomeList: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
