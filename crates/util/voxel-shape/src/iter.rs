//! Iterators for voxel shapes.

use std::iter::FusedIterator;

use voxel_math::BBox;

use crate::{Slice, set};

/// Iterator over all boxes within a voxel shape.
#[derive(Debug)]
pub struct Boxes<'a, 's> {
    pub(crate) slice: &'a Slice<'s>,
    pub(crate) inner: set::iter::Boxes<'a>,
}

impl Iterator for Boxes<'_, '_> {
    type Item = BBox;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(a, b)| {
            BBox::from_raw(
                self.slice.0.__point_pos_vectorized(a.as_usizevec3()),
                self.slice.0.__point_pos_vectorized(b.as_usizevec3()),
            )
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl FusedIterator for Boxes<'_, '_> {}
