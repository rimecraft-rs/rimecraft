use std::{fmt::Debug, hash::Hash};

use glam::{DVec3, IVec3};

use crate::BlockPos;

/// A 3D axis-aligned bounding box.
///
/// The box is defined by its minimum and maximum corners.
#[derive(PartialEq, Clone, Copy)]
#[doc(alias = "AABB")]
pub struct BBox {
    min: DVec3,
    max: DVec3,
}

impl BBox {
    /// Creates a new bounding box from two corners.
    #[inline]
    pub fn new<T1, T2>(p1: T1, p2: T2) -> Self
    where
        T1: Into<DVec3>,
        T2: Into<DVec3>,
    {
        let p1 = p1.into();
        let p2 = p2.into();
        Self {
            min: p1.min(p2),
            max: p1.max(p2),
        }
    }

    /// Creates a new bounding box from raw values:
    /// min `x,y,z`` and max `x,y,z`.
    #[inline]
    pub fn from_raw(min: DVec3, max: DVec3) -> Self {
        debug_assert!(
            min.cmple(max).all(),
            "min must be less than or equal to max"
        );
        Self { min, max }
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
        Self::from_raw(value.0.into(), (value.0 + IVec3::ONE).into())
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
        Self::from_raw(v1.into(), v2.into())
    }
}

impl Debug for BBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BBox")
            .field(&self.min.to_array())
            .field(&self.max.to_array())
            .finish()
    }
}
