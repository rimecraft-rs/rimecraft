//! Minecraft voxel shapes.

pub mod set;

use std::{
    fmt::Debug,
    ops::{Add, Deref, DerefMut},
    sync::{Arc, OnceLock},
};

use glam::{BVec3, UVec3};
use maybe::Maybe;
use parking_lot::Mutex;
use voxel_math::{
    BBox, DVec3,
    direction::{Axis, Direction},
};

pub use set::VoxelSet;

const DOUBLE_BOUNDARY: f64 = 1.0e-7f64;

/// An empty voxel shape slice.
pub fn empty() -> &'static Arc<Slice<'static>> {
    static EMPTY: OnceLock<Arc<Slice<'static>>> = OnceLock::new();
    EMPTY.get_or_init(|| {
        let p: Arc<dyn ErasedList<f64>> = Arc::new(ListEraser([0f64]));
        Array {
            raw: RawVoxelShape {
                voxels: VoxelSet::new((0, 0, 0).into()),
                face_cache: OnceLock::new(),
            },
            xp: p.clone(),
            yp: p.clone(),
            zp: p,
        }
        .into_boxed_slice()
        .into()
    })
}

/// A full cube.
pub fn full_cube() -> &'static Arc<Slice<'static>> {
    static FULL_CUBE: OnceLock<Arc<Slice<'static>>> = OnceLock::new();
    FULL_CUBE.get_or_init(|| {
        Simple(RawVoxelShape {
            voxels: {
                let mut vs = VoxelSet::new((1, 1, 1).into());
                vs.set(0, 0, 0);
                vs
            },
            face_cache: OnceLock::new(),
        })
        .into_boxed_slice()
        .into()
    })
}

pub fn cuboid(bounds: BBox) -> Arc<Slice<'static>> {
    let min = bounds.min();
    let max = bounds.max();

    let fallback = || -> Arc<Slice<'static>> {
        static FALLBACK: OnceLock<Arc<Slice<'static>>> = OnceLock::new();
        FALLBACK
            .get_or_init(|| {
                Array {
                    raw: full_cube().0.__as_raw().clone(),
                    xp: Arc::new(ListEraser([min.x, max.x])),
                    yp: Arc::new(ListEraser([min.y, max.y])),
                    zp: Arc::new(ListEraser([min.z, max.z])),
                }
                .into_boxed_slice()
                .into()
            })
            .clone()
    };

    if (min - max).cmplt(DVec3::splat(DOUBLE_BOUNDARY)).any() {
        empty().clone()
    } else if min.cmplt(DVec3::splat(-DOUBLE_BOUNDARY)).any()
        || max.cmpgt(DVec3::splat(1.0000001f64)).any()
    {
        fallback()
    } else {
        let mut assigned = BVec3::FALSE;
        let mut result = UVec3::ZERO;
        for i in 0..=3u32 {
            let j = (1u32 << i) as f64;
            let d = min * j;
            let e = max * j;
            let v_precision = DVec3::splat(DOUBLE_BOUNDARY * j);
            let bmax = (d - d.round()).abs().cmplt(v_precision);
            let bmin = (e - e.round()).abs().cmplt(v_precision);
            let bmerged = bmax & bmin;
            if i == 0 && bmerged.all() {
                return full_cube().clone();
            }
            let bdiff = bmerged & (assigned ^ bmerged);
            assigned |= bdiff;
            result = UVec3::select(bdiff, UVec3::splat(i), result);
        }

        if assigned.all() {
            let shifted = UVec3::ONE << result;
            let rb_min = (min * shifted.as_dvec3()).round().as_uvec3();
            let rb_max = (max * shifted.as_dvec3()).round().as_uvec3();
            Simple(RawVoxelShape {
                voxels: VoxelSet::with_bounds(
                    shifted.into(),
                    set::Bounds {
                        x: rb_min.x..rb_max.x,
                        y: rb_min.y..rb_max.y,
                        z: rb_min.z..rb_max.z,
                    },
                ),
                face_cache: OnceLock::new(),
            })
            .into_boxed_slice()
            .into()
        } else {
            fallback()
        }
    }
}

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
        (*self).iter().copied()
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
    fn __point_pos(&self, axis: Axis, index: u32) -> f64;

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    );

    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>>;

    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>>;
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
    fn __point_pos_list_arc(&self, axis: Axis) -> Arc<dyn ErasedList<f64>> {
        self.point_pos_list_arc(axis)
    }

    #[inline]
    fn __point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>> {
        self.point_pos_list_boxed(axis)
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
}

/// Slice of a `VoxelShape`.
#[repr(transparent)]
pub struct Slice<'a>(dyn Abstract + Send + Sync + 'a);

impl<'a> Slice<'a> {
    /// An empty slice.
    ///
    /// This is same as function [`empty`].
    #[inline]
    pub fn empty() -> &'a Arc<Self> {
        empty()
    }

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
            .unwrap_or_else(|| Self::empty().clone())
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
    face_cache: OnceLock<Box<Mutex<[Arc<Slice<'static>>; Direction::COUNT]>>>,
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

    #[inline]
    fn point_pos_list_boxed(&self, axis: Axis) -> Box<dyn ErasedList<f64>> {
        Box::new(ListEraser(FractionalDoubleList {
            section_count: self.0.voxels.len_of(axis),
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
pub struct Array {
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

    #[inline]
    fn __as_raw_mut(&mut self) -> &mut RawVoxelShape {
        &mut self.raw
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

impl<'s> SlicedMut<'s, 's> {
    /// Converts the shape into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'s>> {
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
