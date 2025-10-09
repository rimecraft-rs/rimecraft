//! `Heightmap` implementation.

use std::marker::PhantomData;

use rimecraft_block::BlockState;
use rimecraft_packed_int_array::PackedIntArray;
use rimecraft_voxel_math::BlockPos;

use crate::{
    chunk::{BORDER_LEN, ChunkCx},
    view::HeightLimit,
};

const STORAGE_LEN: usize = 256;

/// Maps to store the Y-level of highest block at each horizontal coordinate.
#[derive(Debug)]
pub struct RawHeightmap<'w, P, Cx> {
    storage: PackedIntArray,
    predicate: P,
    hlimit: HeightLimit,
    _marker: PhantomData<&'w Cx>,
}

impl<'w, P, Cx> RawHeightmap<'w, P, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::HeightmapType: Type<'w, Cx, Predicate = P>,
{
    /// Creates a new heightmap.
    ///
    /// # Panics
    ///
    /// Panics when the given `HeightLimit`'s height is more than `2 ^ 256`.
    pub fn new(height_limit: HeightLimit, ty: Cx::HeightmapType) -> Self {
        let i = u32::BITS - (height_limit.height() + 1).leading_zeros();
        Self {
            storage: PackedIntArray::from_packed(i, STORAGE_LEN, None)
                .expect("the height should not more than 2 ^ 256"),
            predicate: ty.predicate(),
            hlimit: height_limit,
            _marker: PhantomData,
        }
    }
}

impl<'w, P, Cx> RawHeightmap<'w, P, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Returns the highest block at the given coordinate.
    #[inline]
    pub fn get(&self, x: i32, z: i32) -> Option<i32> {
        self.storage
            .get(to_index(x, z))
            .map(|y| y as i32 + self.hlimit.bottom())
    }

    /// Sets the highest block at the given coordinate.
    ///
    /// # Panics
    ///
    /// Panics if the given (x, z) coordinate is out of bound.
    #[inline]
    pub fn set(&mut self, x: i32, z: i32, height: i32) {
        self.storage
            .set(to_index(x, z), (height - self.hlimit.bottom()) as u32)
    }
}

impl<'w, P, Cx> RawHeightmap<'w, P, Cx>
where
    Cx: ChunkCx<'w>,
    P: for<'s> FnMut(Option<BlockState<'w, Cx>>) -> bool,
{
    /// Updates this heightmap when the given [`BlockState`] at the location in this map is updated,
    /// and returns whether there is an update to this heightmap.
    pub fn track_update<'a, Pk>(
        &'a mut self,
        x: i32,
        y: i32,
        z: i32,
        state: BlockState<'w, Cx>,
        mut peeker: Pk,
    ) -> bool
    where
        Pk: for<'p> FnMut(BlockPos, &'p mut P) -> bool + 'a,
    {
        let Some(i) = self.get(x, z).filter(|i| y > *i - 2) else {
            return false;
        };

        if (self.predicate)(Some(state)) {
            if y >= i {
                self.set(x, z, y + 1);
                true
            } else {
                false
            }
        } else if y == i - 1 {
            for j in (self.hlimit.bottom()..y).rev() {
                if peeker(BlockPos::new(x, j, z), &mut self.predicate) {
                    self.set(x, z, j + 1);
                    return true;
                }
            }

            self.set(x, z, self.hlimit.bottom());
            true
        } else {
            false
        }
    }
}

#[inline]
const fn to_index(x: i32, z: i32) -> usize {
    (x + z * BORDER_LEN as i32) as usize
}

/// Several different heightmaps check and store highest block of different types,
/// and are used for different purposes.
pub trait Type<'w, Cx: ChunkCx<'w>>: 'w {
    /// Predicate of block states.
    type Predicate: for<'s> Fn(Option<BlockState<'w, Cx>>) -> bool;

    /// Predicate of this type.
    fn predicate(&self) -> Self::Predicate;

    /// Returns an [`Iterator`] of this type, containing all types that is required
    /// to be updated on block state updates in `WorldChunk`.
    fn iter_block_update_types_wc() -> impl Iterator<Item = &'w Self>;
}

/// [`RawHeightmap`] with predicate type filled with [`Type::Predicate`].
pub type Heightmap<'w, Cx> =
    RawHeightmap<'w, <<Cx as ChunkCx<'w>>::HeightmapType as Type<'w, Cx>>::Predicate, Cx>;
