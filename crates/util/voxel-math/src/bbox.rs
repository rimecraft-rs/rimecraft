use std::hash::Hash;

use glam::{DVec3, IVec3};

use crate::BlockPos;

/// A 3D axis-aligned bounding box.
///
/// The box is defined by its minimum and maximum corners.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BBox {
    min: DVec3,
    max: DVec3,
}

impl BBox {
    /// Creates a new bounding box from two corners.
    pub fn new(p1: DVec3, p2: DVec3) -> Self {
        Self {
            min: p1.min(p2),
            max: p1.max(p2),
        }
    }

    /// Gets the minimum corner of the box.
    #[inline]
    pub fn min(&self) -> DVec3 {
        self.min
    }

    /// Gets the maximum corner of the box.
    #[inline]
    pub fn max(&self) -> DVec3 {
        self.max
    }
}

impl Hash for BBox {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let slice: &[f64; 3] = self.min.as_ref();
        for s in slice {
            s.to_bits().hash(state);
        }
        let slice: &[f64; 3] = self.max.as_ref();
        for s in slice {
            s.to_bits().hash(state);
        }
    }
}

impl Eq for BBox {}

impl From<(DVec3, DVec3)> for BBox {
    #[inline]
    fn from((min, max): (DVec3, DVec3)) -> Self {
        Self::new(min, max)
    }
}

impl From<BlockPos> for BBox {
    #[inline]
    fn from(value: BlockPos) -> Self {
        Self::new(value.0.into(), (value.0 + IVec3::ONE).into())
    }
}

impl From<DVec3> for BBox {
    #[inline]
    fn from(value: DVec3) -> Self {
        Self::new(value, value + DVec3::ONE)
    }
}

impl From<(BlockPos, BlockPos)> for BBox {
    fn from((p1, p2): (BlockPos, BlockPos)) -> Self {
        let v1 = p1.0.min(p2.0);
        let v2 = p1.0.max(p2.0) + IVec3::ONE;
        Self::new(v1.into(), v2.into())
    }
}
