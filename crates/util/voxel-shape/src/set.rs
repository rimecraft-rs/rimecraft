//! Voxel sets.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Range},
    sync::Arc,
};

use bitvec::{bitbox, boxed::BitBox, slice::BitSlice};
use maybe::Maybe;
use voxel_math::direction::Axis;

trait Abstract {
    fn __props(&self) -> Props;

    fn __bitslice(&self) -> &BitSlice;
    fn __bitslice_mut(&mut self) -> &mut BitSlice;

    fn __set_ext(&mut self, x: u32, y: u32, z: u32);
    fn __index(&self, x: u32, y: u32, z: u32) -> usize;

    fn __bounds(&self, axis: Axis) -> Range<u32>;

    fn __is_empty(&self) -> bool {
        for axis in Axis::VALUES {
            if self.__bounds(axis).is_empty() {
                return true;
            }
        }
        false
    }

    fn __contains(&self, x: u32, y: u32, z: u32) -> bool {
        self.__bitslice()
            .get(self.__index(x, y, z))
            .as_deref()
            .copied()
            .unwrap_or_default()
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
}

/// Slice of a voxel set.
#[repr(transparent)]
pub struct Slice<'s>(dyn Abstract + Send + Sync + 's);

impl Debug for Slice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxelSetSlice")
            .field("inner_slice", &self.0.__bitslice())
            .field("props", &self.0.__props())
            .finish()
    }
}

impl<'s> Slice<'s> {
    /// Whether this set contains a voxel at the given position.
    #[inline]
    pub fn contains(&self, x: u32, y: u32, z: u32) -> bool {
        self.0.__contains(x, y, z)
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
}

#[allow(unsafe_code)] // SAFETY: safe because the type is marked as `repr(transparent)`
impl<'a> Slice<'a> {
    #[inline]
    fn from_ref<'s>(this: &'s (dyn Abstract + Send + Sync + 'a)) -> &'s Self {
        unsafe { std::mem::transmute(this) }
    }

    #[inline]
    fn from_mut<'s>(this: &'s mut (dyn Abstract + Send + Sync + 'a)) -> &'s mut Self {
        unsafe { std::mem::transmute(this) }
    }

    #[inline]
    fn from_boxed(this: Box<dyn Abstract + Send + Sync + 'a>) -> Box<Self> {
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

/// A voxel set implemented using a bit set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoxelSet {
    props: Props,
    bounds: Bounds,
    data: BitBox,
}

impl VoxelSet {
    /// Creates a new voxel set.
    #[inline]
    pub fn new(props: Props) -> Self {
        let Props {
            len_x,
            len_y,
            len_z,
        } = props;
        let len = (props.len_x as usize) * (props.len_y as usize) * (props.len_z as usize);
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

    /// Creates a new voxel set with the given bounds.
    pub fn with_bounds(props: Props, bounds: Bounds) -> Self {
        let mut this = Self::new(props);
        this.bounds = bounds;
        this
    }

    /// Converts this set into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<Slice<'static>> {
        Slice::from_boxed(Box::new(self))
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
        ((x * self.props.len_y + y) * self.props.len_z + z) as usize
    }

    #[inline]
    fn __bounds(&self, axis: Axis) -> Range<u32> {
        axis.choose(
            self.bounds.x.start,
            self.bounds.y.start,
            self.bounds.z.start,
        )..axis.choose(self.bounds.x.end, self.bounds.y.end, self.bounds.z.end)
    }

    #[inline]
    fn __is_empty(&self) -> bool {
        self.data.not_any()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxed() {
        let mut set = VoxelSet::new(Props {
            len_x: 16,
            len_y: 16,
            len_z: 16,
        });

        assert!(!set.contains(1, 5, 4));
        set.set(1, 5, 4);
        assert!(set.contains(1, 5, 4));
    }

    #[test]
    fn crop() {
        let mut set = VoxelSet::new(Props {
            len_x: 16,
            len_y: 16,
            len_z: 16,
        });
        set.set(8, 8, 8);

        let mut cropped = set.crop_mut(Bounds {
            x: 4..12,
            y: 4..12,
            z: 4..12,
        });
        assert!(cropped.contains(4, 4, 4));
        cropped.set(1, 3, 5);

        assert!(set.contains(5, 7, 9));
    }
}
