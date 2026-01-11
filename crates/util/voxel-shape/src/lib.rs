//! Minecraft voxel shapes.

mod func;
pub mod iter;
mod list;
pub mod set;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

use approx::abs_diff_eq;
use glam::{DVec3, USizeVec3, UVec3};
use maybe::Maybe;

pub use func::*;
use remap::remap_method;
pub use set::VoxelSet;
pub use voxel_math;

use voxel_math::{
    BBox, BlockHitResult, BlockPos,
    direction::{Axis, AxisDirection, Direction},
};

use crate::{
    iter::Boxes,
    list::{ErasedList, FractionalDoubleList, List, ListDeref, ListEraser, OffsetList},
};

pub use crate::Slice as VoxelShapeSlice;
pub use set::Slice as VoxelSetSlice;

const F64_TOLERANCE: f64 = 1.0e-7f64;
const MAX_SHAPE_RESOLUTION: u32 = 8;

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

    fn __point_pos_vectorized(&self, index: USizeVec3) -> DVec3 {
        DVec3::new(
            self.__point_pos(Axis::X, index.x),
            self.__point_pos(Axis::Y, index.y),
            self.__point_pos(Axis::Z, index.z),
        )
    }

    fn __iter_point_pos(
        &self,
        axis: Axis,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_),
    );

    fn __point_pos_len(&self, axis: Axis) -> usize;

    fn __point_pos_len_vectorized(&self) -> USizeVec3 {
        USizeVec3::new(
            self.__point_pos_len(Axis::X),
            self.__point_pos_len(Axis::Y),
            self.__point_pos_len(Axis::Z),
        )
    }

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

    fn __min_vectorized(&self) -> DVec3 {
        let voxels = &self.__as_raw().voxels;
        let i = UVec3::from_array(voxels.0.__bounds_vectorized().into_array().map(|r| r.start));

        DVec3::select(
            i.cmpge(voxels.0.__len_vectorized()),
            self.__point_pos_vectorized(i.as_usizevec3()),
            DVec3::splat(f64::INFINITY),
        )
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

    fn __max_vectorized(&self) -> DVec3 {
        let voxels = &self.__as_raw().voxels;
        let i = UVec3::from_array(voxels.0.__bounds_vectorized().into_array().map(|r| r.end));

        DVec3::select(
            i.cmpge(voxels.0.__len_vectorized()),
            self.__point_pos_vectorized(i.as_usizevec3()),
            DVec3::splat(f64::NEG_INFINITY),
        )
    }

    #[inline]
    fn __is_empty(&self) -> bool {
        self.__as_raw().voxels.is_empty()
    }

    fn __bounding_box(&self) -> BBox {
        assert!(!self.__is_empty(), "no bounds for empty shape");
        BBox::from_raw(self.__min_vectorized(), self.__max_vectorized())
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
        self.__point_pos_len_vectorized()
            .cmpeq(USizeVec3::splat(2))
            .all()
            && self
                .__point_pos_vectorized(USizeVec3::splat(0))
                .abs_diff_eq(DVec3::splat(0.0), F64_TOLERANCE)
            && self
                .__point_pos_vectorized(USizeVec3::splat(1))
                .abs_diff_eq(DVec3::splat(1.0), F64_TOLERANCE)
    }

    fn __priv_is_square(&self, axis: Axis) -> bool {
        self.__point_pos_len(axis) == 2
            && abs_diff_eq!(self.__point_pos(axis, 0), 0f64, epsilon = F64_TOLERANCE)
            && abs_diff_eq!(self.__point_pos(axis, 1), 1f64, epsilon = F64_TOLERANCE)
    }

    fn __face(&self, this: &Arc<Slice<'static>>, facing: Direction) -> Option<Arc<Slice<'static>>> {
        // None for itself
        debug_assert!(
            std::ptr::addr_eq(Arc::as_ptr(this), self),
            "this must be identical to self"
        );

        if !self.__is_empty() && !std::ptr::addr_eq(self, Arc::as_ptr(full_cube())) {
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

    fn __priv_coord_index_vectorized(&self, coord: DVec3) -> UVec3 {
        UVec3 {
            x: self.__priv_coord_index(Axis::X, coord.x),
            y: self.__priv_coord_index(Axis::Y, coord.y),
            z: self.__priv_coord_index(Axis::Z, coord.z),
        }
    }
}

/// Slice of a voxel shape.
#[repr(transparent)]
#[derive(Debug)]
#[doc(alias = "VoxelShapeSlice")]
pub struct Slice<'a>(dyn Abstract + 'a);

fn slice_set_bounds(set: &set::Slice<'_>, axis: Axis, width: u32) -> set::Bounds {
    let (sx, sy, sz) = set.0.__len_vectorized().into();
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

    /// Restructures this voxel shape by merging largest coalesce boxes' cuboid shapes into one large shape.
    pub fn simplify(&self) -> Arc<Self> {
        self.boxes()
            .map(cuboid)
            .reduce(|a, b| combine(&a, &b))
            .unwrap_or_else(|| empty().clone())
    }

    /// Iterates over all boxes within a voxel shape.
    pub fn boxes(&self) -> Boxes<'_, 'a> {
        Boxes {
            slice: self,
            inner: self.0.__as_raw().voxels.boxes(),
        }
    }

    /// Raycasts through this voxel shape, offseted by the given block position.
    ///
    /// See [`voxel_math::raycast_block`] for more information.
    #[remap_method(yarn = "raycast", mojmaps = "clip")]
    pub fn raycast(&self, src: DVec3, dst: DVec3, pos: BlockPos) -> Option<BlockHitResult> {
        if self.is_empty() {
            return None;
        }
        let delta = dst - src;
        if delta.length_squared() < F64_TOLERANCE {
            return None;
        }

        let adj = src + delta * 0.001;
        let p = self.0.__priv_coord_index_vectorized(adj - DVec3::from(pos));
        if self
            .0
            .__as_raw()
            .voxels
            .in_bounds_and_contains(p.x, p.y, p.z)
        {
            Some(BlockHitResult::new(
                adj,
                Direction::from(delta).opposite(),
                pos,
                true,
            ))
        } else {
            voxel_math::raycast_block(self.boxes(), src, dst, pos)
        }
    }

    /// Iterates each boxes in this slice.
    #[deprecated = "use `boxes` to get an iterator instead. this method now only serves the need to remind people"]
    #[remap_method(yarn = "forEachBox", mojmaps = "forAllBoxes")]
    pub fn for_each_box<F>(&self, f: F)
    where
        F: FnMut(BBox),
    {
        self.boxes().for_each(f);
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
    #[inline]
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

    #[inline]
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
        assert!(
            std::ptr::addr_eq(&self.parent.0, Arc::as_ptr(&arc)),
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
