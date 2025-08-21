//! Voxel sets.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Range},
    sync::Arc,
};

use bitvec::{bitbox, boxed::BitBox, slice::BitSlice};
use glam::UVec3;
use maybe::Maybe;
use voxel_math::direction::Axis;

use crate::{
    list::{PairErasedList, PairListIterItem},
    set::iter::{Boxes, Voxels},
};

pub mod iter;

pub(crate) trait Abstract: Send + Sync + Debug {
    fn __props(&self) -> Props;

    fn __bitslice(&self) -> &BitSlice;
    fn __bitslice_mut(&mut self) -> &mut BitSlice;

    fn __set_ext(&mut self, x: u32, y: u32, z: u32);
    fn __index(&self, x: u32, y: u32, z: u32) -> usize;

    fn __bounds(&self, axis: Axis) -> Range<u32>;

    fn __bounds_vectorized(&self) -> Bounds {
        Bounds {
            x: self.__bounds(Axis::X),
            y: self.__bounds(Axis::Y),
            z: self.__bounds(Axis::Z),
        }
    }

    fn __is_empty(&self) -> bool {
        self.__bounds_vectorized()
            .into_array()
            .iter()
            .any(Range::is_empty)
    }

    fn __contains(&self, x: u32, y: u32, z: u32) -> bool {
        self.__bitslice()
            .get(self.__index(x, y, z))
            .as_deref()
            .copied()
            .unwrap_or_default()
    }

    fn __in_bounds_and_contains(&self, x: u32, y: u32, z: u32) -> bool {
        let props = self.__props();
        x < props.len_x && y < props.len_y && z < props.len_z && self.__contains(x, y, z)
    }

    fn __set(&mut self, x: u32, y: u32, z: u32, perform_ext: bool) {
        let index = self.__index(x, y, z);
        self.__bitslice_mut().set(index, true);
        if perform_ext {
            self.__set_ext(x, y, z);
        }
    }

    fn __len_of(&self, axis: Axis) -> u32 {
        let Props {
            len_x,
            len_y,
            len_z,
        } = self.__props();
        axis.choose(len_x, len_y, len_z)
    }

    fn __len_vectorized(&self) -> UVec3 {
        let Props {
            len_x,
            len_y,
            len_z,
        } = self.__props();
        (len_x, len_y, len_z).into()
    }

    #[inline]
    fn __bits_data(&self) -> Option<&BitSlice> {
        None
    }
}

/// Slice of a voxel set.
#[repr(transparent)]
#[doc(alias = "VoxelSetSlice")]
#[derive(Debug)]
pub struct Slice<'s>(pub(crate) dyn Abstract + 's);

impl<'s> Slice<'s> {
    /// Whether this set contains a voxel at the given position.
    ///
    /// See [`Self::in_bounds_and_contains`] for bounds checking.
    #[inline]
    pub fn contains(&self, x: u32, y: u32, z: u32) -> bool {
        self.0.__contains(x, y, z)
    }

    /// Whether this set contains a voxel at the given position which is within the set's overall length.
    #[inline]
    pub fn in_bounds_and_contains(&self, x: u32, y: u32, z: u32) -> bool {
        self.0.__in_bounds_and_contains(x, y, z)
    }

    /// Sets the voxel at given position.
    #[inline]
    pub fn set(&mut self, x: u32, y: u32, z: u32) {
        self.0.__set(x, y, z, true);
    }

    /// Gets the length of given axis.
    #[inline]
    pub fn len_of(&self, axis: Axis) -> u32 {
        self.0.__len_of(axis)
    }

    /// Gets the bounds of give axis.
    #[inline]
    pub fn bounds_of(&self, axis: Axis) -> Range<u32> {
        self.0.__bounds(axis)
    }

    /// Crops this set into a cropped slice.
    ///
    /// See [`Self::crop_arc`] for cropping by reference counting.
    pub fn crop<'a>(&'a self, bounds: Bounds) -> Cropped<'a, 's> {
        Cropped {
            props: Props {
                len_x: bounds.x.end - bounds.x.start,
                len_y: bounds.y.end - bounds.y.start,
                len_z: bounds.z.end - bounds.z.start,
            },
            bounds,
            parent: Maybe::Borrowed(self),
        }
    }

    /// Crops this set into a cropped slice with reference counted.
    ///
    /// See [`Self::crop`] for cropping by lifetimed reference.
    pub fn crop_arc(self: &Arc<Self>, bounds: Bounds) -> Cropped<'static, 's> {
        Cropped {
            props: Props {
                len_x: bounds.x.end - bounds.x.start,
                len_y: bounds.y.end - bounds.y.start,
                len_z: bounds.z.end - bounds.z.start,
            },
            bounds,
            parent: Maybe::Owned(self.clone()),
        }
    }

    /// Crops this set into a mutable cropped slice.
    pub fn crop_mut<'a>(&'a mut self, bounds: Bounds) -> CroppedMut<'a, 's> {
        CroppedMut {
            props: Props {
                len_x: bounds.x.end - bounds.x.start,
                len_y: bounds.y.end - bounds.y.start,
                len_z: bounds.z.end - bounds.z.start,
            },
            bounds,
            parent: self,
        }
    }

    /// Returns the _inclusive_ minimum coordinate of the set along the given axis.
    #[inline]
    pub fn min(&self, axis: Axis) -> u32 {
        self.bounds_of(axis).start
    }

    /// Returns the _exclusive_ maximum coordinate of the set along the given axis.
    #[inline]
    pub fn max(&self, axis: Axis) -> u32 {
        self.bounds_of(axis).end
    }

    /// Returns whether the underlying slice is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.__is_empty()
    }

    /// Returns an iterator over the coalesced boxes of the set.
    #[inline]
    pub fn boxes(&self) -> Boxes<'_> {
        Boxes::from_slice(self)
    }

    /// Returns an iterator over every single voxels of the set.
    #[inline]
    pub fn voxels(&self) -> Voxels<'_, 's> {
        Voxels::from_slice(self)
    }
}

#[allow(unsafe_code)] // SAFETY: safe because the type is marked as `repr(transparent)`
impl<'a> Slice<'a> {
    #[inline]
    fn from_ref<'s>(this: &'s (dyn Abstract + 'a)) -> &'s Self {
        unsafe { std::mem::transmute(this) }
    }

    #[inline]
    fn from_mut<'s>(this: &'s mut (dyn Abstract + 'a)) -> &'s mut Self {
        unsafe { std::mem::transmute(this) }
    }

    #[inline]
    fn from_boxed(this: Box<dyn Abstract + 'a>) -> Box<Self> {
        unsafe { std::mem::transmute(this) }
    }
}

/// Basic properties of a voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// Don't use `repr(packed)`: it's discouraged by rustc.
pub struct Props {
    /// Length of the set in the X direction.
    pub len_x: u32,
    /// Length of the set in the Y direction.
    pub len_y: u32,
    /// Length of the set in the Z direction.
    pub len_z: u32,
}

impl From<(u32, u32, u32)> for Props {
    #[inline]
    fn from((len_x, len_y, len_z): (u32, u32, u32)) -> Self {
        Self {
            len_x,
            len_y,
            len_z,
        }
    }
}

impl From<UVec3> for Props {
    #[inline]
    fn from(v: UVec3) -> Self {
        <(u32, u32, u32)>::from(v).into()
    }
}

/// Bounds of a voxel set.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bounds {
    /// The range of X values that are set.
    pub x: Range<u32>,
    /// The range of Y values that are set.
    pub y: Range<u32>,
    /// The range of Z values that are set.
    pub z: Range<u32>,
}

impl Bounds {
    #[inline]
    pub(crate) fn into_array(self) -> [Range<u32>; 3] {
        [self.x, self.y, self.z]
    }
}

/// A voxel set implemented using a bit set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoxelSet {
    props: Props,
    bounds: Bounds,
    data: BitBox,
}

impl VoxelSet {
    /// Creates a new voxel set without filling it.
    #[inline]
    pub fn new(props: Props) -> Self {
        let Props {
            len_x,
            len_y,
            len_z,
        } = props;
        let len = (props.len_x * props.len_y * props.len_z) as usize;
        Self {
            props,
            bounds: Bounds {
                x: 0..len_x,
                y: 0..len_y,
                z: 0..len_z,
            },
            data: bitbox![0; len],
        }
    }

    /// Creates a new voxel set with the given bounds, filling bounded area.
    pub fn with_bounds(props: Props, bounds: Bounds) -> Self {
        let mut this = Self::new(props);
        this.bounds = bounds;
        // cheap clones
        for x in this.bounds.x.clone() {
            for y in this.bounds.y.clone() {
                for z in this.bounds.z.clone() {
                    this.set(x, y, z);
                }
            }
        }
        this
    }

    /// Creates a new bitvec-backed voxel set from a slice, accepts any underlying representation.
    pub fn from_slice(slice: &Slice<'_>) -> Self {
        let value = slice;
        let props = value.0.__props();

        Self {
            props,
            bounds: value.0.__bounds_vectorized(),
            data: value.0.__bits_data().map_or_else(
                || {
                    let mut bits = bitbox![0; (props.len_x * props.len_y * props.len_z) as usize];
                    for x in 0..props.len_x {
                        for y in 0..props.len_y {
                            for z in 0..props.len_z {
                                if value.contains(x, y, z) {
                                    bits.set(__vset_index(props, x, y, z), true);
                                }
                            }
                        }
                    }
                    bits
                },
                BitBox::from_bitslice,
            ),
        }
    }

    /// Converts this set into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'static>> {
        Slice::from_boxed(Box::new(self))
    }

    fn is_column_full(&self, x: u32, y: u32, z: Range<u32>) -> bool {
        x < self.props.len_x
            && y < self.props.len_y
            && self.data
                [__vset_index(self.props, x, y, z.start)..__vset_index(self.props, x, y, z.end)]
                .all()
    }

    fn clear_column(&mut self, x: u32, y: u32, z: Range<u32>) {
        debug_assert!(x < self.props.len_x, "x should be less than len_x");
        debug_assert!(y < self.props.len_y, "y should be less than len_y");

        self.data[__vset_index(self.props, x, y, z.start)..__vset_index(self.props, x, y, z.end)]
            .fill(false);
    }

    pub(crate) fn combine_with<F>(
        lhs: &Slice<'_>,
        rhs: &Slice<'_>,
        points: [&(dyn PairErasedList<f64> + '_); 3],
        f: F,
    ) -> Self
    where
        F: Fn(bool, bool) -> bool,
    {
        let (xp, yp, zp) = (points[0], points[1], points[2]);
        let mut bounds_cast = [u32::MAX, u32::MAX, u32::MAX, u32::MIN, u32::MIN, u32::MIN];
        let mut this = Self::new(Props {
            len_x: xp.__erased_len() as u32 - 1,
            len_y: yp.__erased_len() as u32 - 1,
            len_z: zp.__erased_len() as u32 - 1,
        });

        xp.__peek_pair_erased_iter(&mut |xit| {
            for PairListIterItem {
                x: x1,
                y: x2,
                index: xi,
            } in xit
            {
                let mut x_active = false;
                let (x1, x2, xi) = (x1 as u32, x2 as u32, xi as u32);

                yp.__peek_pair_erased_iter(&mut |yit| {
                    for PairListIterItem {
                        x: y1,
                        y: y2,
                        index: yi,
                    } in yit
                    {
                        let mut y_active = false;
                        let (y1, y2, yi) = (y1 as u32, y2 as u32, yi as u32);

                        zp.__peek_pair_erased_iter(&mut |zit| {
                            for PairListIterItem {
                                x: z1,
                                y: z2,
                                index: zi,
                            } in zit
                            {
                                let (z1, z2, zi) = (z1 as u32, z2 as u32, zi as u32);
                                if f(
                                    lhs.in_bounds_and_contains(x1, y1, z1),
                                    rhs.in_bounds_and_contains(x2, y2, z2),
                                ) {
                                    this.data.set(__vset_index(this.props, xi, yi, zi), true);

                                    bounds_cast[2] = bounds_cast[2].min(zi);
                                    bounds_cast[5] = bounds_cast[5].max(zi);
                                    y_active = true;
                                }
                            }
                        });

                        if y_active {
                            bounds_cast[1] = bounds_cast[1].min(yi);
                            bounds_cast[4] = bounds_cast[4].max(yi);
                            x_active = true;
                        }
                    }
                });

                if x_active {
                    bounds_cast[0] = bounds_cast[0].min(xi);
                    bounds_cast[3] = bounds_cast[3].max(xi);
                }
            }
        });

        this.bounds = Bounds {
            x: bounds_cast[0]..bounds_cast[3] + 1,
            y: bounds_cast[1]..bounds_cast[4] + 1,
            z: bounds_cast[2]..bounds_cast[5] + 1,
        };

        this
    }
}

impl Abstract for VoxelSet {
    #[inline]
    fn __props(&self) -> Props {
        self.props
    }

    #[inline]
    fn __bitslice(&self) -> &BitSlice {
        &self.data
    }

    #[inline]
    fn __bitslice_mut(&mut self) -> &mut BitSlice {
        &mut self.data
    }

    #[inline]
    fn __set_ext(&mut self, x: u32, y: u32, z: u32) {
        macro_rules! se {
            ($($a:ident),*$(,)?) => {
                $(self.bounds.$a =
                    self.bounds.$a.start.min($a)..self.bounds.$a.end.max($a + 1);)*
            };
        }
        se![x, y, z];
    }

    #[inline]
    fn __index(&self, x: u32, y: u32, z: u32) -> usize {
        __vset_index(self.props, x, y, z)
    }

    #[inline]
    fn __bounds(&self, axis: Axis) -> Range<u32> {
        axis.choose(&self.bounds.x, &self.bounds.y, &self.bounds.z)
            .clone()
    }

    #[inline]
    fn __is_empty(&self) -> bool {
        self.data.not_any()
    }

    #[inline]
    fn __bits_data(&self) -> Option<&BitSlice> {
        Some(&self.data)
    }
}

#[inline]
fn __vset_index(props: Props, x: u32, y: u32, z: u32) -> usize {
    // continuous inside a column
    ((x * props.len_y + y) * props.len_z + z) as usize
}

impl Deref for VoxelSet {
    type Target = Slice<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl DerefMut for VoxelSet {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Slice::from_mut(self)
    }
}

impl From<&Slice<'_>> for VoxelSet {
    #[inline]
    fn from(value: &Slice<'_>) -> Self {
        Self::from_slice(value)
    }
}

/// A cropped voxel set.
#[derive(Debug, Clone)]
pub struct Cropped<'a, 's> {
    props: Props,
    bounds: Bounds,
    parent: Maybe<'a, Slice<'s>, Arc<Slice<'s>>>,
}

impl<'a> Cropped<'a, '_> {
    /// Converts this set into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl Abstract for Cropped<'_, '_> {
    #[inline]
    fn __props(&self) -> Props {
        self.props
    }

    #[inline]
    fn __bitslice(&self) -> &BitSlice {
        self.parent.0.__bitslice()
    }

    #[inline]
    fn __bitslice_mut(&mut self) -> &mut BitSlice {
        unreachable!("cropped is only for immutable usage")
    }

    #[inline]
    fn __set_ext(&mut self, _x: u32, _y: u32, _z: u32) {
        unreachable!("cropped is only for immutable usage")
    }

    #[inline]
    fn __index(&self, x: u32, y: u32, z: u32) -> usize {
        self.parent.0.__index(
            self.bounds.x.start + x,
            self.bounds.y.start + y,
            self.bounds.z.start + z,
        )
    }

    #[inline]
    fn __bounds(&self, axis: Axis) -> Range<u32> {
        let i = axis.choose(
            self.bounds.x.start,
            self.bounds.y.start,
            self.bounds.z.start,
        );
        let j = axis.choose(self.bounds.x.end, self.bounds.y.end, self.bounds.z.end);
        let bounds = self.parent.0.__bounds(axis);
        bounds.start.clamp(i, j) - i..bounds.end.clamp(i, j) - i
    }
}

/// A mutable cropped voxel set.
#[derive(Debug)]
pub struct CroppedMut<'a, 's> {
    props: Props,
    bounds: Bounds,
    parent: &'a mut Slice<'s>,
}

impl<'a> CroppedMut<'a, '_> {
    /// Converts this set into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'a>> {
        Slice::from_boxed(Box::new(self))
    }
}

impl Abstract for CroppedMut<'_, '_> {
    #[inline]
    fn __props(&self) -> Props {
        self.props
    }

    #[inline]
    fn __bitslice(&self) -> &BitSlice {
        self.parent.0.__bitslice()
    }

    #[inline]
    fn __bitslice_mut(&mut self) -> &mut BitSlice {
        self.parent.0.__bitslice_mut()
    }

    #[inline]
    fn __set_ext(&mut self, x: u32, y: u32, z: u32) {
        self.parent.0.__set_ext(
            self.bounds.x.start + x,
            self.bounds.y.start + y,
            self.bounds.z.start + z,
        )
    }

    #[inline]
    fn __index(&self, x: u32, y: u32, z: u32) -> usize {
        self.parent.0.__index(
            self.bounds.x.start + x,
            self.bounds.y.start + y,
            self.bounds.z.start + z,
        )
    }

    #[inline]
    fn __bounds(&self, axis: Axis) -> Range<u32> {
        let i = axis.choose(
            self.bounds.x.start,
            self.bounds.y.start,
            self.bounds.z.start,
        );
        let j = axis.choose(self.bounds.x.end, self.bounds.y.end, self.bounds.z.end);
        let bounds = self.parent.0.__bounds(axis);
        bounds.start.clamp(i, j) - i..bounds.end.clamp(i, j) - i
    }
}

impl<'a> Deref for Cropped<'a, '_> {
    type Target = Slice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl<'a> Deref for CroppedMut<'a, '_> {
    type Target = Slice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Slice::from_ref(self)
    }
}

impl DerefMut for CroppedMut<'_, '_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Slice::from_mut(self)
    }
}
