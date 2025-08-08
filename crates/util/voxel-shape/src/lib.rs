//! Minecraft voxel shapes.

pub mod set;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use voxel_math::direction::Axis;

pub use set::VoxelSet;

trait Abstract {
    fn __as_raw(&self) -> &RawVoxelShape;
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape;

    fn __index_point_pos(&self, axis: Axis, index: u32) -> Option<f64>;
    fn __point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a>;

    fn __min(&self, axis: Axis) -> f64 {
        let voxels = &self.__as_raw().voxels;
        let i = voxels.bounds_of(axis).start;

        (i >= voxels.len_of(axis))
            .then(|| self.__index_point_pos(axis, i))
            .flatten()
            .unwrap_or(f64::INFINITY)
    }

    fn __max(&self, axis: Axis) -> f64 {
        let voxels = &self.__as_raw().voxels;
        let i = voxels.bounds_of(axis).end;

        (i >= voxels.len_of(axis))
            .then(|| self.__index_point_pos(axis, i))
            .flatten()
            .unwrap_or(f64::NEG_INFINITY)
    }
}

/// Slice of a `VoxelShape`.
#[repr(transparent)]
pub struct Slice<'a>(dyn Abstract + Send + Sync + 'a);

impl Slice<'_> {
    /// Returns the minimum coordinate of the shape along the given axis.
    #[inline]
    pub fn min(&self, axis: Axis) -> f64 {
        self.0.__min(axis)
    }

    /// Returns the maximum coordinate of the shape along the given axis.
    #[inline]
    pub fn max(&self, axis: Axis) -> f64 {
        self.0.__max(axis)
    }
}

impl Debug for Slice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxelShapeSlice")
            .field("voxels", &self.0.__as_raw().voxels)
            .finish()
    }
}

#[allow(unsafe_code)] // SAFETY: repr(transparent)
impl<'a> Slice<'a> {
    #[inline]
    fn from_ref<'s>(shape: &'s (dyn Abstract + Send + Sync + 'a)) -> &'s Self {
        unsafe { std::mem::transmute(shape) }
    }

    #[inline]
    fn from_mut<'s>(shape: &'s mut (dyn Abstract + Send + Sync + 'a)) -> &'s mut Self {
        unsafe { std::mem::transmute(shape) }
    }

    #[inline]
    fn from_boxed(shape: Box<dyn Abstract + Send + Sync + 'a>) -> Box<Self> {
        unsafe { std::mem::transmute(shape) }
    }
}

#[derive(Debug, Clone)]
struct RawVoxelShape {
    voxels: VoxelSet,
    shape_cache: Vec<Arc<Slice<'static>>>, //TODO: done these
}

/// A simple voxel shape.
#[derive(Debug, Clone)]
pub struct Simple(RawVoxelShape);

impl Simple {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'static>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl Abstract for Simple {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.0
    }

    #[inline]
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.0
    }

    fn __index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        let len = self.0.voxels.len_of(axis);
        if len == 0 {
            None
        } else {
            Some((index as f64) / len as f64)
        }
    }

    fn __point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
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
    type Target = Slice<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl DerefMut for Simple {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Slice::from_mut(self)
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
    pub fn into_boxed_slice(self) -> Box<Slice<'static>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl Abstract for Array {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.raw
    }

    #[inline]
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.raw
    }

    fn __index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        let arr = match axis {
            Axis::X => &self.xp,
            Axis::Y => &self.yp,
            Axis::Z => &self.zp,
        };
        arr.get(index as usize).copied()
    }

    fn __point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        let arr = match axis {
            Axis::X => &self.xp,
            Axis::Y => &self.yp,
            Axis::Z => &self.zp,
        };
        Box::new(arr.iter().copied())
    }
}

impl Deref for Array {
    type Target = Slice<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl DerefMut for Array {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Slice::from_mut(self)
    }
}

/// A voxel shape that is a slice of another shape.
#[derive(Debug, Clone)]
pub struct Sliced<'a, 's> {
    parent: &'a Slice<'s>,
    shape: RawVoxelShape,
    axis: Axis,
}

impl<'a> Sliced<'a, 'a> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl Abstract for Sliced<'_, '_> {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.shape
    }

    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        unreachable!("Sliced shape is immutable")
    }

    fn __index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        if axis == self.axis {
            Some(index as f64)
        } else {
            self.parent.0.__index_point_pos(axis, index)
        }
    }

    fn __point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        if axis == self.axis {
            Box::new((0u32..=1).map(|i| i as f64))
        } else {
            self.parent.0.__point_poss(axis)
        }
    }
}

/// A mutable voxel shape that is a slice of another shape.
#[derive(Debug)]
pub struct SlicedMut<'a, 's> {
    parent: &'a mut Slice<'s>,
    shape: RawVoxelShape,
    axis: Axis,
}

impl<'a> SlicedMut<'a, 'a> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl Abstract for SlicedMut<'_, '_> {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.shape
    }

    #[inline]
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.shape
    }

    fn __index_point_pos(&self, axis: Axis, index: u32) -> Option<f64> {
        if axis == self.axis {
            Some(index as f64)
        } else {
            self.parent.0.__index_point_pos(axis, index)
        }
    }

    fn __point_poss<'a>(&'a self, axis: Axis) -> Box<dyn Iterator<Item = f64> + 'a> {
        if axis == self.axis {
            Box::new((0u32..=1).map(|i| i as f64))
        } else {
            self.parent.0.__point_poss(axis)
        }
    }
}

impl<'s> Deref for Sliced<'s, 's> {
    type Target = Slice<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl<'s> Deref for SlicedMut<'s, 's> {
    type Target = Slice<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl<'s> DerefMut for SlicedMut<'s, 's> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Slice::from_mut(self)
    }
}
