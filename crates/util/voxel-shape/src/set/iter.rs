//! Iterators for voxel sets.

use std::{iter::FusedIterator, marker::PhantomData};

use glam::UVec3;

use crate::{
    VoxelSet,
    set::{Props, Slice},
};

enum ControlFlow<T> {
    Continue,
    Break,
    Return(T),
}

/// Iterator over all boxes in a voxel set.
///
/// Boxes are represented as a pair of coordinates. This iterator is *coalesce*:
/// it will coalesce adjacent voxels into a single box if possible.
///
/// See [`Slice::boxes`] for obtaining this iterator.
#[derive(Debug, Clone)]
pub struct Boxes<'a> {
    set: VoxelSet,

    x: u32,
    y: u32,
    z: u32,

    k: Option<u32>,

    _marker: PhantomData<&'a ()>,
}

impl Boxes<'_> {
    pub(super) fn from_slice(slice: &Slice<'_>) -> Self {
        Self {
            set: slice.into(),
            x: 0,
            y: 0,
            z: 0,
            k: None,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn __next(&mut self) -> ControlFlow<(UVec3, UVec3)> {
        // z-axis iterates inclusive while others are exclusive
        // order: y -> x -> z
        if self.z > self.set.props.len_z {
            self.z = 0;
            self.x += 1;
            self.k = None;
        }
        if self.x >= self.set.props.len_x {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.set.props.len_y {
            return ControlFlow::Break;
        }

        let (x, y, z) = (self.x, self.y, self.z);

        if self.set.in_bounds_and_contains(x, y, z) {
            self.k = self.k.or(Some(z));
            return ControlFlow::Continue;
        }
        let Some(k) = self.k else {
            return ControlFlow::Continue;
        };

        let mut m = x;
        let mut n = y;
        self.set.clear_column(x, y, k..z);

        while self.set.is_column_full(m + 1, y, k..z) {
            self.set.clear_column(m + 1, y, k..z);
            m += 1;
        }

        while (x..=m).all(|i| self.set.is_column_full(i, n + 1, k..z)) {
            (x..=m).for_each(|i| self.set.clear_column(i, n + 1, k..z));
            n += 1;
        }

        self.k = None;
        ControlFlow::Return(((x, y, k).into(), (m + 1, n + 1, z).into()))
    }
}

impl Iterator for Boxes<'_> {
    type Item = (UVec3, UVec3);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = self.__next();
            self.z += 1;
            return match result {
                ControlFlow::Continue => continue,
                ControlFlow::Break => None,
                ControlFlow::Return(tuple) => Some(tuple),
            };
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ones = self.set.data.count_ones();
        (if ones == 0 { 0 } else { 1 }, Some(ones))
    }
}

impl FusedIterator for Boxes<'_> {}

/// An iterator over the voxels of a voxel set.
///
/// This is similar to [`Boxes`], but treats every single voxel as a box.
#[derive(Debug, Clone)]
pub struct Voxels<'a, 's> {
    slice: &'a Slice<'s>,
    props: Props,

    x: u32,
    y: u32,
    z: u32,
}

impl<'a, 's> Voxels<'a, 's> {
    pub(super) fn from_slice(slice: &'a Slice<'s>) -> Self {
        Self {
            slice,
            props: slice.0.__props(),
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Voxels<'_, '_> {
    #[inline]
    fn __next(&mut self) -> ControlFlow<(UVec3, UVec3)> {
        if self.z >= self.props.len_z {
            self.z = 0;
            self.x += 1;
        }
        if self.x >= self.props.len_x {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.props.len_y {
            return ControlFlow::Break;
        }

        let (x, y, z) = (self.x, self.y, self.z);

        if self.slice.in_bounds_and_contains(x, y, z) {
            ControlFlow::Return(((x, y, z).into(), (x + 1, y + 1, z + 1).into()))
        } else {
            ControlFlow::Continue
        }
    }
}

impl Iterator for Voxels<'_, '_> {
    type Item = (UVec3, UVec3);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = self.__next();
            self.z += 1;
            return match result {
                ControlFlow::Continue => continue,
                ControlFlow::Break => None,
                ControlFlow::Return(tuple) => Some(tuple),
            };
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ones = self.slice.0.__bits_data().map_or_else(
            || self.slice.0.__len_vectorized().element_product() as usize,
            bitvec::slice::BitSlice::count_ones,
        );
        (if ones == 0 { 0 } else { 1 }, Some(ones))
    }
}

impl FusedIterator for Voxels<'_, '_> {}
