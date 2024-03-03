//! Minecraft voxel shapes.

pub mod set;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use rimecraft_voxel_math::direction::Axis;
use set::VoxelSet;

trait AbstVoxelShape {
    fn as_raw(&self) -> &RawVoxelShape;
    fn as_raw_mut(&mut self) -> &mut RawVoxelShape;

    fn index_point_pos(&self, axis: Axis, index: u32) -> Option<f64>;
    fn point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a>;
}

/// Slice of a `VoxelShape`.
#[repr(transparent)]
pub struct VoxelShapeSlice<'a> {
    inner: dyn AbstVoxelShape + Send + Sync + 'a,
}

impl VoxelShapeSlice<'_> {
    /// Returns the minimum coordinate of the shape along the given axis.
    pub fn min(&self, axis: Axis) -> f64 {
        let voxels = &self.inner.as_raw().voxels;
        let i = *voxels.bounds_of(axis).start();

        (i >= voxels.len_of(axis))
            .then(|| self.inner.index_point_pos(axis, i))
            .flatten()
            .unwrap_or(f64::INFINITY)
    }

    /// Returns the maximum coordinate of the shape along the given axis.
    pub fn max(&self, axis: Axis) -> f64 {
        let voxels = &self.inner.as_raw().voxels;
        let i = *voxels.bounds_of(axis).end();

        (i >= voxels.len_of(axis))
            .then(|| self.inner.index_point_pos(axis, i))
            .flatten()
            .unwrap_or(f64::NEG_INFINITY)
    }
}

impl Debug for VoxelShapeSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxelShapeSlice")
            .field("voxels", &self.inner.as_raw().voxels)
            .finish()
    }
}

#[allow(unsafe_code)] // SAFETY: repr(transparent)
impl<'a> VoxelShapeSlice<'a> {
    #[inline]
    fn from_ref<'s>(shape: &'s (dyn AbstVoxelShape + Send + Sync + 'a)) -> &'s Self {
        unsafe { std::mem::transmute(shape) }
    }

    #[inline]
    fn from_mut<'s>(shape: &'s mut (dyn AbstVoxelShape + Send + Sync + 'a)) -> &'s mut Self {
        unsafe { std::mem::transmute(shape) }
    }

    #[inline]
    fn from_boxed(shape: Box<dyn AbstVoxelShape + Send + Sync + 'a>) -> Box<Self> {
        unsafe { std::mem::transmute(shape) }
    }
}

#[derive(Debug, Clone)]
struct RawVoxelShape {
    voxels: VoxelSet,
    shape_cache: Vec<Arc<VoxelShapeSlice<'static>>>,
}

/// A simple voxel shape.
#[derive(Debug, Clone)]
pub struct Simple(RawVoxelShape);

impl Simple {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<VoxelShapeSlice<'static>> {
        VoxelShapeSlice::from_boxed(Box::new(self))
    }
}

impl AbstVoxelShape for Simple {
    #[inline]
    fn as_raw(&self) -> &RawVoxelShape {
        &self.0
    }

    #[inline]
    fn as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.0
    }

    fn index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        let len = self.0.voxels.len_of(axis);
        if len == 0 {
            None
        } else {
            Some((index as f64) / len as f64)
        }
    }

    fn point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        let len = self.0.voxels.len_of(axis);
        if len == 0 {
            Box::new(std::iter::empty())
        } else {
            let len2 = len as f64;
            Box::new((0u32..=len).map(move |index| index as f64 / len2))
        }
    }
}

impl Deref for Simple {
    type Target = VoxelShapeSlice<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelShapeSlice::from_ref(self)
    }
}

impl DerefMut for Simple {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        VoxelShapeSlice::from_mut(self)
    }
}

/// A voxel shape that is a backed by point arrays.
#[derive(Debug, Clone)]
pub struct Array {
    raw: RawVoxelShape,

    xp: Box<[f64]>,
    yp: Box<[f64]>,
    zp: Box<[f64]>,
}

impl Array {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<VoxelShapeSlice<'static>> {
        VoxelShapeSlice::from_boxed(Box::new(self))
    }
}

impl AbstVoxelShape for Array {
    #[inline]
    fn as_raw(&self) -> &RawVoxelShape {
        &self.raw
    }

    #[inline]
    fn as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.raw
    }

    fn index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        let arr = match axis {
            Axis::X => &self.xp,
            Axis::Y => &self.yp,
            Axis::Z => &self.zp,
        };
        arr.get(index as usize).copied()
    }

    fn point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        let arr = match axis {
            Axis::X => &self.xp,
            Axis::Y => &self.yp,
            Axis::Z => &self.zp,
        };
        Box::new(arr.iter().copied())
    }
}

impl Deref for Array {
    type Target = VoxelShapeSlice<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelShapeSlice::from_ref(self)
    }
}

impl DerefMut for Array {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        VoxelShapeSlice::from_mut(self)
    }
}

/// A voxel shape that is a slice of another shape.
#[derive(Debug, Clone)]
pub struct Sliced<'a, 's> {
    parent: &'a VoxelShapeSlice<'s>,
    shape: RawVoxelShape,
    axis: Axis,
}

impl<'a> Sliced<'a, 'a> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<VoxelShapeSlice<'a>> {
        VoxelShapeSlice::from_boxed(Box::new(self))
    }
}

impl AbstVoxelShape for Sliced<'_, '_> {
    #[inline]
    fn as_raw(&self) -> &RawVoxelShape {
        &self.shape
    }

    fn as_raw_mut(&mut self) -> &mut RawVoxelShape {
        unreachable!("Sliced shape is immutable")
    }

    fn index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        if axis == self.axis {
            Some(index as f64)
        } else {
            self.parent.inner.index_point_pos(axis, index)
        }
    }

    fn point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        if axis == self.axis {
            Box::new((0u32..=1).map(|i| i as f64))
        } else {
            self.parent.inner.point_poss(axis)
        }
    }
}

/// A mutable voxel shape that is a slice of another shape.
#[derive(Debug)]
pub struct SlicedMut<'a, 's> {
    parent: &'a mut VoxelShapeSlice<'s>,
    shape: RawVoxelShape,
    axis: Axis,
}

impl<'a> SlicedMut<'a, 'a> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<VoxelShapeSlice<'a>> {
        VoxelShapeSlice::from_boxed(Box::new(self))
    }
}

impl AbstVoxelShape for SlicedMut<'_, '_> {
    #[inline]
    fn as_raw(&self) -> &RawVoxelShape {
        &self.shape
    }

    #[inline]
    fn as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.shape
    }

    fn index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        if axis == self.axis {
            Some(index as f64)
        } else {
            self.parent.inner.index_point_pos(axis, index)
        }
    }

    fn point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        if axis == self.axis {
            Box::new((0u32..=1).map(|i| i as f64))
        } else {
            self.parent.inner.point_poss(axis)
        }
    }
}

impl<'s> Deref for Sliced<'s, 's> {
    type Target = VoxelShapeSlice<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelShapeSlice::from_ref(self)
    }
}

impl<'s> Deref for SlicedMut<'s, 's> {
    type Target = VoxelShapeSlice<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelShapeSlice::from_ref(self)
    }
}

impl<'s> DerefMut for SlicedMut<'s, 's> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        VoxelShapeSlice::from_mut(self)
    }
}
