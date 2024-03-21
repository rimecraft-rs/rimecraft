//! `Heightmap` implementation.

use std::marker::PhantomData;

use rimecraft_block::BlockState;
use rimecraft_packed_int_array::PackedIntArray;

use crate::{chunk::ChunkCx, view::HeightLimit};

const STORAGE_LEN: usize = 256;

/// Maps to store the Y-level of highest block at each horizontal coordinate.
#[derive(Debug)]
pub struct RawHeightmap<'w, P, Cx> {
    storage: PackedIntArray,
    predicate: P,
    _marker: PhantomData<fn(&'w Cx)>,
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
    pub fn new(hlimit: HeightLimit, ty: Cx::HeightmapType) -> Self {
        let i = u32::BITS - (hlimit.height() + 1).leading_zeros();
        Self {
            storage: PackedIntArray::from_packed(i, STORAGE_LEN, None)
                .expect("the height should not more than 2 ^ 256"),
            predicate: ty.predicate(),
            _marker: PhantomData,
        }
    }
}

/// Several different heightmaps check and store highest block of different types,
/// and are used for different purposes.
pub trait Type<'w, Cx: ChunkCx<'w>> {
    /// Predicate of block states.
    type Predicate: for<'s> Fn(&'s BlockState<'w, Cx>) -> bool;

    /// Predicate of this type.
    fn predicate(&self) -> Self::Predicate;
}

/// [`RawHeightmap`] with predicate type filled with [`Type::Predicate`].
pub type Heightmap<'w, Cx> =
    RawHeightmap<'w, <<Cx as ChunkCx<'w>>::HeightmapType as Type<'w, Cx>>::Predicate, Cx>;
