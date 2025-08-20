//! Minecraft voxel shapes.

mod func;
pub mod set;

use std::{
    fmt::Debug,
    ops::{Add, Deref, DerefMut},
    sync::{Arc, OnceLock},
};

use approx::abs_diff_eq;
use maybe::Maybe;
use voxel_math::{
    BBox, DVec3,
    direction::{Axis, AxisDirection, Direction},
};

pub use func::*;
pub use set::VoxelSet;

pub use crate::Slice as VoxelShapeSlice;
pub use set::Slice as VoxelSetSlice;

const DOUBLE_BOUNDARY: f64 = 1.0e-7f64;
const MAX_SHAPE_RESOLUTION: u32 = 8;

trait List<T> {
    fn index(&self, index: usize) -> T;
    #[allow(unused)] // myth bug
    fn iter(&self) -> impl IntoIterator<Item = T>;
    fn len(&self) -> usize;
}

trait ErasedList<T>: Send + Sync + Debug {
    fn __erased_index(&self, index: usize) -> T;

    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_));

    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a;

    fn __erased_len(&self) -> usize;
}

impl<T> List<T> for [T]
where
    T: Copy,
{
    #[inline]
    fn index(&self, index: usize) -> T {
        self[index]
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        <[T]>::iter(self).copied()
    }

    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> List<T> for [T; N]
where
    T: Copy,
{
    #[inline]
    fn index(&self, index: usize) -> T {
        self[index]
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        self.as_slice().iter().copied()
    }

    #[inline]
    fn len(&self) -> usize {
        N
    }
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

    #[inline]
    fn len(&self) -> usize {
        (**self).len()
    }
}

#[derive(Debug, Clone, Copy)]
struct FractionalDoubleList {
    section_count: usize,
}

impl List<f64> for FractionalDoubleList {
    #[inline]
    fn index(&self, index: usize) -> f64 {
        index as f64 / self.section_count as f64
    }

    fn iter(&self) -> impl IntoIterator<Item = f64> {
        (0..=self.section_count).map(|i| i as f64 / self.section_count as f64)
    }

    fn len(&self) -> usize {
        self.section_count + 1
    }
}

#[repr(transparent)]
struct ListDeref<T>(T);

impl<T, I> ErasedList<I> for ListDeref<T>
where
    T: Deref<Target: ErasedList<I>> + Send + Sync + Debug,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> I {
        self.0.__erased_index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = I> + '_)) + '_)) {
        self.0.__peek_erased_iter(f);
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a>
    where
        I: 'a,
    {
        self.0.__boxed_erased_iter()
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.0.__erased_len()
    }
}

#[repr(transparent)]
struct ListEraser<T: ?Sized>(T);

impl<L, T> ErasedList<T> for ListEraser<L>
where
    L: List<T> + Send + Sync + Debug + ?Sized,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> T {
        self.0.index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_)) {
        f(&mut self.0.iter().into_iter())
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a,
    {
        Box::new(self.0.iter().into_iter())
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Debug + ?Sized> Debug for ListEraser<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Debug> Debug for ListDeref<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
struct OffsetList<I, T: ?Sized> {
    offset: I,
    inner: T,
}

impl<I, T> ErasedList<I> for OffsetList<I, T>
where
    T: ErasedList<I>,
    I: Add<Output = I> + Copy + Send + Sync + Debug,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> I {
        self.inner.__erased_index(index) + self.offset
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = I> + '_)) + '_)) {
        self.inner
            .__peek_erased_iter(&mut |iter| f(&mut iter.map(|i| i + self.offset)));
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a>
    where
        I: 'a,
    {
        Box::new(self.inner.__boxed_erased_iter().map(|i| i + self.offset))
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.inner.__erased_len()
    }
}

impl<I, T> From<(I, T)> for OffsetList<I, T> {
    #[inline(always)]
    fn from(value: (I, T)) -> Self {
        Self {
            offset: value.0,
            inner: value.1,
        }
    }
}

trait ProvidePointPosList {
    fn point_pos_list(&self, axis: Axis) -> impl List<f64> + Send + Sync + Debug + '_;
    fn point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>>;

    #[inline]
    fn point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>> {
        Arc::from(self.point_pos_list_boxed(axis))
    }
}

trait ErasedProvidePointPosList: Send + Sync + Debug {
    fn __point_pos(&self, axis: Axis, index: usize) -> f64;

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    );

    fn __point_pos_len(&self, axis: Axis) -> usize;

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>>;
    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>>;
}

impl<P> ErasedProvidePointPosList for P
where
    P: ProvidePointPosList + Send + Sync + Debug,
{
    #[inline]
    fn __point_pos(&self, axis: Axis, index: usize) -> f64 {
        self.point_pos_list(axis).index(index)
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
    fn __point_pos_len(&self, axis: Axis) -> usize {
        self.point_pos_list(axis).len()
    }

    #[inline]
    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>> {
        self.point_pos_list_arc(axis)
    }

    #[inline]
    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>> {
        self.point_pos_list_boxed(axis)
    }
}

trait Abstract: ErasedProvidePointPosList + Send + Sync + Debug {
    fn __as_raw(&self) -> &RawVoxelShape;

    fn __min(&self, axis: Axis) -> f64 {
        let voxels = &self.__as_raw().voxels;
        let i = voxels.bounds_of(axis).start;

        if i >= voxels.len_of(axis) {
            self.__point_pos(axis, i as usize)
        } else {
            f64::INFINITY
        }
    }

    fn __max(&self, axis: Axis) -> f64 {
        let voxels = &self.__as_raw().voxels;
        let i = voxels.bounds_of(axis).end;

        if i >= voxels.len_of(axis) {
            self.__point_pos(axis, i as usize)
        } else {
            f64::NEG_INFINITY
        }
    }

    #[inline]
    fn __is_empty(&self) -> bool {
        self.__as_raw().voxels.is_empty()
    }

    fn __bounding_box(&self) -> BBox {
        assert!(!self.__is_empty(), "no bounds for empty shape");
        BBox::from_raw(
            DVec3 {
                x: self.__min(Axis::X),
                y: self.__min(Axis::Y),
                z: self.__min(Axis::Z),
            },
            DVec3 {
                x: self.__max(Axis::X),
                y: self.__max(Axis::Y),
                z: self.__min(Axis::Z),
            },
        )
    }

    fn __offset(&self, DVec3 { x, y, z }: DVec3) -> Option<Array> {
        if self.__is_empty() {
            None
        } else {
            Some(Array {
                raw: self.__as_raw().clone(),
                xp: Arc::new(OffsetList::from((
                    x,
                    ListDeref(self.__point_pos_list_arc(Axis::X)),
                ))),
                yp: Arc::new(OffsetList::from((
                    y,
                    ListDeref(self.__point_pos_list_arc(Axis::Y)),
                ))),
                zp: Arc::new(OffsetList::from((
                    z,
                    ListDeref(self.__point_pos_list_arc(Axis::Z)),
                ))),
            })
        }
    }

    fn __priv_is_cube(&self) -> bool {
        Axis::VALUES.map(|a| self.__priv_is_square(a)) == [true; 3]
    }

    fn __priv_is_square(&self, axis: Axis) -> bool {
        self.__point_pos_len(axis) == 2
            && abs_diff_eq!(self.__point_pos(axis, 0), 0f64, epsilon = 1.0e-7)
            && abs_diff_eq!(self.__point_pos(axis, 1), 1f64, epsilon = 1.0e-7)
    }

    #[allow(clippy::if_then_some_else_none)]
    fn __face(&self, this: &Arc<Slice<'static>>, facing: Direction) -> Option<Arc<Slice<'static>>> {
        // None for itself
        debug_assert_eq!(
            Arc::as_ptr(this).cast::<()>(),
            std::ptr::from_ref(self).cast::<()>(),
            "this must be identical to self"
        );

        if !self.__is_empty()
            && std::ptr::from_ref(self).cast::<()>() != Arc::as_ptr(full_cube()).cast::<()>()
        {
            let cache = &self
                .__as_raw()
                .face_cache
                .get_or_init(|| Box::new([const { OnceLock::new() }; Direction::COUNT]))
                [facing.ordinal()];

            cache
                .get_or_init(|| {
                    let axis = facing.axis();
                    if self.__priv_is_square(axis) {
                        return None;
                    }
                    let axis_dir = facing.axis_dir();
                    let i = self.__priv_coord_index(
                        axis,
                        if axis_dir == AxisDirection::Positive {
                            0.9999999f64
                        } else {
                            1e-7f64
                        },
                    );
                    let s = this.slice(axis, i);
                    Some(if s.is_empty() {
                        empty().clone()
                    } else if self.__priv_is_cube() {
                        full_cube().clone()
                    } else {
                        s.adopt_arc(this.clone()).into_boxed_slice().into()
                    })
                })
                .clone()
        } else {
            None
        }
    }

    fn __priv_coord_index(&self, axis: Axis, coord: f64) -> u32 {
        let max = self.__as_raw().voxels.len_of(axis) + 1;
        math::binary_search_ie_u32(0..max, |i| coord < self.__point_pos(axis, i as usize))
            .unwrap_or(max)
            - 1
    }
}

/// Slice of a voxel shape.
#[repr(transparent)]
#[derive(Debug)]
#[doc(alias = "VoxelShapeSlice")]
pub struct Slice<'a>(dyn Abstract + 'a);
fn slice_set_bounds(set: &set::Slice<'_>, axis: Axis, width: u32) -> set::Bounds {
    let (sx, sy, sz) = (
        set.len_of(Axis::X),
        set.len_of(Axis::Y),
        set.len_of(Axis::Z),
    );
    set::Bounds {
        x: axis.choose(width, 0, 0)..axis.choose(width + 1, sy, sz),
        y: axis.choose(0, width, 0)..axis.choose(sx, width + 1, sz),
        z: axis.choose(0, 0, width)..axis.choose(sx, sy, width + 1),
    }
}

impl<'a> Slice<'a> {
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

    /// Returns whether this shape is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.__is_empty()
    }

    /// Returns the minimum [`BBox`] containing this shape.
    ///
    /// # Panics
    ///
    /// - Panics if this shape is empty.
    #[inline]
    pub fn bounding_box(&self) -> BBox {
        self.0.__bounding_box()
    }

    /// Returns a shape that is offset by the given vector.
    ///
    /// This function's implementation is highly costy.
    pub fn offset<P>(&self, offset: P) -> Arc<Self>
    where
        P: Into<DVec3>,
    {
        self.0
            .__offset(offset.into())
            .map(|a| Arc::from(a.into_boxed_slice()))
            .unwrap_or_else(|| empty().clone())
    }

    /// Returns a shape that is sliced along the given axis.
    ///
    /// See [`Self::slice_arc`] for reference-counted version.
    pub fn slice(&self, axis: Axis, width: u32) -> Sliced<'_, 'a> {
        let raw = self.0.__as_raw();
        Sliced {
            parent: Maybe::Borrowed(self),
            sliced_shape: RawVoxelShape::from_arc(
                raw.voxels
                    .crop_arc(slice_set_bounds(&raw.voxels, axis, width))
                    .into_boxed_slice()
                    .into(),
            ),
            axis,
        }
    }

    /// Returns a shape that is sliced along the given axis with a reference-counted lifetime.
    ///
    /// See [`Self::slice`] for borrowed version.
    pub fn slice_arc(self: &Arc<Self>, axis: Axis, width: u32) -> Sliced<'static, 'a> {
        let raw = self.0.__as_raw();
        Sliced {
            parent: Maybe::Owned(self.clone()),
            sliced_shape: RawVoxelShape::from_arc(
                raw.voxels
                    .crop_arc(slice_set_bounds(&raw.voxels, axis, width))
                    .into_boxed_slice()
                    .into(),
            ),
            axis,
        }
    }

    /// Gets the position of a specific point in the given axis.
    #[inline]
    pub fn point(&self, axis: Axis, index: usize) -> f64 {
        self.0.__point_pos(axis, index)
    }

    /// Peeks an iterator over the point positions in the given axis.
    #[allow(clippy::missing_panics_doc)] // panic points unreachable
    pub fn peek_iter_points<F, T>(&self, axis: Axis, f: F) -> T
    where
        F: FnOnce(&mut (dyn Iterator<Item = f64> + '_)) -> T,
    {
        let mut opt = None;
        let mut f = Some(f);
        self.0.__iter_point_pos(axis, &mut |iter| {
            if let Some(f) = f.take() {
                opt = Some(f(iter));
            }
        });
        opt.unwrap()
    }
}

impl Slice<'static> {
    /// Returns face of this slice in the given direction.
    pub fn face(self: &Arc<Self>, direction: Direction) -> Arc<Self> {
        self.0
            .__face(self, direction)
            .unwrap_or_else(|| self.clone())
    }
}

#[allow(unsafe_code)] // SAFETY: repr(transparent)
impl<'a> Slice<'a> {
    #[inline]
    fn from_ref<'s>(shape: &'s (dyn Abstract + 'a)) -> &'s Self {
        unsafe { std::mem::transmute(shape) }
    }

    #[inline]
    fn from_mut<'s>(shape: &'s mut (dyn Abstract + 'a)) -> &'s mut Self {
        unsafe { std::mem::transmute(shape) }
    }

    #[inline]
    fn from_boxed(shape: Box<dyn Abstract + 'a>) -> Box<Self> {
        unsafe { std::mem::transmute(shape) }
    }
}

#[allow(clippy::type_complexity)]
struct RawVoxelShape {
    voxels: Arc<set::Slice<'static>>,
    //WIP
    face_cache: OnceLock<Box<[OnceLock<Option<Arc<Slice<'static>>>>; Direction::COUNT]>>,
}

impl RawVoxelShape {
    #[inline]
    fn from_arc(voxels: Arc<set::Slice<'static>>) -> Self {
        Self {
            voxels,
            face_cache: OnceLock::new(),
        }
    }
}

impl Debug for RawVoxelShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&*self.voxels, f)
    }
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
struct Simple(RawVoxelShape);

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

    fn __priv_coord_index(&self, axis: Axis, coord: f64) -> u32 {
        let i = self.0.voxels.len_of(axis) as f64;
        (coord * i).clamp(-1f64, i).floor() as u32
    }
}

impl ProvidePointPosList for Simple {
    #[inline]
    fn point_pos_list(&self, axis: Axis) -> impl List<f64> + Send + Sync + Debug + '_ {
        FractionalDoubleList {
            section_count: self.0.voxels.len_of(axis) as usize,
        }
    }

    #[inline]
    fn point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>> {
        Box::new(ListEraser(FractionalDoubleList {
            section_count: self.0.voxels.len_of(axis) as usize,
        }))
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
struct Array {
    raw: RawVoxelShape,

    xp: Arc<dyn ErasedList<f64>>,
    yp: Arc<dyn ErasedList<f64>>,
    zp: Arc<dyn ErasedList<f64>>,
}

impl Array {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'static>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl ErasedProvidePointPosList for Array {
    fn __point_pos(&self, axis: Axis, index: usize) -> f64 {
        axis.choose(&*self.xp, &*self.yp, &*self.zp)
            .__erased_index(index)
    }

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    ) {
        axis.choose(&*self.xp, &*self.yp, &*self.zp)
            .__peek_erased_iter(f);
    }

    fn __point_pos_len(&self, axis: Axis) -> usize {
        axis.choose(&self.xp, &self.yp, &self.zp).__erased_len()
    }

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>> {
        axis.choose(&self.xp, &self.yp, &self.zp).clone()
    }

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>> {
        Box::new(ListDeref(self.__point_pos_list_arc(axis)))
    }
}

impl Abstract for Array {
    #[inline]
    fn __as_raw(&self) -> &RawVoxelShape {
        &self.raw
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
    parent: Maybe<'a, Slice<'s>, Arc<Slice<'s>>>,
    sliced_shape: RawVoxelShape,
    axis: Axis,
}

impl<'s> Sliced<'s, 's> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'s>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl<'s> Sliced<'_, 's> {
    /// Transforms this sliced shape into reference-counted internally.
    ///
    /// # Panics
    ///
    /// Panics if the given slice is not identical to the one this shape is holding.
    #[inline]
    pub fn adopt_arc(self, arc: Arc<Slice<'s>>) -> Sliced<'static, 's> {
        assert_eq!(
            std::ptr::from_ref(&self.parent.0).cast::<()>(),
            Arc::as_ptr(&arc).cast::<()>(),
            "arc must be the same as the parent slice"
        );
        Sliced {
            parent: Maybe::Owned(arc),
            sliced_shape: self.sliced_shape,
            axis: self.axis,
        }
    }
}

const SINGULAR_FRACTIONAL: FractionalDoubleList = FractionalDoubleList { section_count: 1 };

impl ErasedProvidePointPosList for Sliced<'_, '_> {
    fn __point_pos(&self, axis: Axis, index: usize) -> f64 {
        if self.axis == axis {
            SINGULAR_FRACTIONAL.index(index)
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

    fn __point_pos_len(&self, axis: Axis) -> usize {
        if self.axis == axis {
            SINGULAR_FRACTIONAL.len()
        } else {
            self.parent.0.__point_pos_len(axis)
        }
    }

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>> {
        if self.axis == axis {
            Arc::new(ListEraser(SINGULAR_FRACTIONAL))
        } else {
            self.parent.0.__point_pos_list_arc(axis)
        }
    }

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>> {
        if self.axis == axis {
            Box::new(ListEraser(SINGULAR_FRACTIONAL))
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
}

impl<'s> Deref for Sliced<'s, 's> {
    type Target = Slice<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

#[cfg(test)]
mod tests;
