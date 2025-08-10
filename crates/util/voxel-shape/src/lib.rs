//! Minecraft voxel shapes.

pub mod set;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;
use voxel_math::direction::{Axis, Direction};

pub use set::VoxelSet;

trait List<T> {
    fn index(&self, index: usize) -> T;
    fn iter(&self) -> impl IntoIterator<Item = T>;
}

trait ErasedList<T>: Send + Sync + Debug {
    fn __erased_index(&self, index: usize) -> T;

    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_));

    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a;
}

impl<L, T> List<T> for &L
where
    L: List<T> + ?Sized,
{
    #[inline]
    fn index(&self, index: usize) -> T {
        (**self).index(index)
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        (**self).iter()
    }
}

#[derive(Debug, Clone, Copy)]
struct FractionalDoubleList {
    section_count: u32,
}

impl List<f64> for FractionalDoubleList {
    #[inline]
    fn index(&self, index: usize) -> f64 {
        index as f64 / self.section_count as f64
    }

    fn iter(&self) -> impl IntoIterator<Item = f64> {
        (0..=self.section_count).map(|i| i as f64 / self.section_count as f64)
    }
}

impl<T> List<T> for dyn ErasedList<T> + '_ {
    #[inline]
    fn index(&self, index: usize) -> T {
        self.__erased_index(index)
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        self.__boxed_erased_iter()
    }
}

impl<L, T> ErasedList<T> for L
where
    L: List<T> + Send + Sync + Debug + ?Sized,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> T {
        self.index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_)) {
        f(&mut self.iter().into_iter())
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a,
    {
        Box::new(self.iter().into_iter())
    }
}

trait ProvidePointPosList {
    fn point_pos_list(&self, axis: Axis) -> impl List<f64> + Send + Sync + Debug + '_;
    fn point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64> + '_> {
        Arc::new(self.point_pos_list(axis))
    }
}

trait ErasedProvidePointPosList: Send + Sync + Debug {
    fn __point_pos(&self, axis: Axis, index: u32) -> f64;

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    );

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64> + '_>;

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64> + '_>;
}

impl<P> ErasedProvidePointPosList for P
where
    P: ProvidePointPosList + Send + Sync + Debug,
{
    #[inline]
    fn __point_pos(&self, axis: Axis, index: u32) -> f64 {
        self.point_pos_list(axis).index(index as usize)
    }

    #[inline]
    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    ) {
        f(&mut self.point_pos_list(axis).iter().into_iter())
    }

    #[inline]
    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64> + '_> {
        self.point_pos_list_arc(axis)
    }

    #[inline]
    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64> + '_> {
        Box::new(self.point_pos_list(axis))
    }
}

trait Abstract: ErasedProvidePointPosList {
    fn __as_raw(&self) -> &RawVoxelShape;
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape;

    fn __min(&self, axis: Axis) -> f64 {
        let voxels = &self.__as_raw().voxels;
        let i = voxels.bounds_of(axis).start;

        if i >= voxels.len_of(axis) {
            self.__point_pos(axis, i)
        } else {
            f64::INFINITY
        }
    }

    fn __max(&self, axis: Axis) -> f64 {
        let voxels = &self.__as_raw().voxels;
        let i = voxels.bounds_of(axis).end;

        if i >= voxels.len_of(axis) {
            self.__point_pos(axis, i)
        } else {
            f64::NEG_INFINITY
        }
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

#[derive(Debug)]
struct RawVoxelShape {
    voxels: VoxelSet,
    //WIP
    face_cache: OnceLock<Mutex<[Arc<()>; Direction::COUNT]>>,
}

impl Clone for RawVoxelShape {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            voxels: self.voxels.clone(),
            face_cache: OnceLock::new(),
        }
    }
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
}

impl ProvidePointPosList for Simple {
    #[inline]
    fn point_pos_list(&self, axis: Axis) -> impl List<f64> + Send + Sync + Debug + '_ {
        FractionalDoubleList {
            section_count: self.0.voxels.len_of(axis),
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
pub struct Array<'a> {
    raw: RawVoxelShape,

    xp: Arc<dyn ErasedList<f64> + 'a>,
    yp: Arc<dyn ErasedList<f64> + 'a>,
    zp: Arc<dyn ErasedList<f64> + 'a>,
}

impl<'a> Array<'a> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl ErasedProvidePointPosList for Array<'_> {
    fn __point_pos(&self, axis: Axis, index: u32) -> f64 {
        axis.choose(&*self.xp, &*self.yp, &*self.zp)
            .__erased_index(index as usize)
    }

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    ) {
        axis.choose(&*self.xp, &*self.yp, &*self.zp)
            .__peek_erased_iter(f);
    }

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64> + '_> {
        axis.choose(&self.xp, &self.yp, &self.zp).clone()
    }

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64> + '_> {
        Box::new(axis.choose(&*self.xp, &*self.yp, &*self.zp))
    }
}

impl Abstract for Array<'_> {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.raw
    }

    #[inline]
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.raw
    }
}

impl<'a> Deref for Array<'a> {
    type Target = Slice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl DerefMut for Array<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Slice::from_mut(self)
    }
}

/// A voxel shape that is a slice of another shape.
#[derive(Debug, Clone)]
pub struct Sliced<'a, 's> {
    parent: &'a Slice<'s>,
    sliced_shape: RawVoxelShape,
    axis: Axis,
}

impl<'a> Sliced<'a, '_> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

const SINGULAR_FRACTIONAL: FractionalDoubleList = FractionalDoubleList { section_count: 1 };

impl ErasedProvidePointPosList for Sliced<'_, '_> {
    fn __point_pos(&self, axis: Axis, index: u32) -> f64 {
        if self.axis == axis {
            SINGULAR_FRACTIONAL.index(index as usize)
        } else {
            self.parent.0.__point_pos(axis, index)
        }
    }

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    ) {
        if self.axis == axis {
            f(&mut SINGULAR_FRACTIONAL.iter().into_iter())
        } else {
            self.parent.0.__iter_point_pos(axis, f)
        }
    }

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64> + '_> {
        if self.axis == axis {
            Arc::new(SINGULAR_FRACTIONAL)
        } else {
            self.parent.0.__point_pos_list_arc(axis)
        }
    }

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64> + '_> {
        if self.axis == axis {
            Box::new(SINGULAR_FRACTIONAL)
        } else {
            self.parent.0.__point_pos_list_boxed(axis)
        }
    }
}

impl Abstract for Sliced<'_, '_> {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.sliced_shape
    }

    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        unreachable!("Sliced shape is immutable")
    }
}

/// A mutable voxel shape that is a slice of another shape.
#[derive(Debug)]
pub struct SlicedMut<'a, 's> {
    parent: &'a mut Slice<'s>,
    sliced_shape: RawVoxelShape,
    axis: Axis,
}

impl<'a> SlicedMut<'a, 'a> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl ErasedProvidePointPosList for SlicedMut<'_, '_> {
    fn __point_pos(&self, axis: Axis, index: u32) -> f64 {
        if self.axis == axis {
            SINGULAR_FRACTIONAL.index(index as usize)
        } else {
            self.parent.0.__point_pos(axis, index)
        }
    }

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    ) {
        if self.axis == axis {
            f(&mut SINGULAR_FRACTIONAL.iter().into_iter())
        } else {
            self.parent.0.__iter_point_pos(axis, f)
        }
    }

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64> + '_> {
        if self.axis == axis {
            Arc::new(SINGULAR_FRACTIONAL)
        } else {
            self.parent.0.__point_pos_list_arc(axis)
        }
    }

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64> + '_> {
        if self.axis == axis {
            Box::new(SINGULAR_FRACTIONAL)
        } else {
            self.parent.0.__point_pos_list_boxed(axis)
        }
    }
}

impl Abstract for SlicedMut<'_, '_> {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.sliced_shape
    }

    #[inline]
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.sliced_shape
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
