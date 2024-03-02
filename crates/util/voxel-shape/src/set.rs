//! Voxel sets.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, RangeInclusive},
};

use bitvec::{bitbox, boxed::BitBox, slice::BitSlice};
use rimecraft_voxel_math::direction::Axis;

trait AsVoxelSet {
    fn props(&self) -> Props;

    fn bitslice(&self) -> &BitSlice;
    fn bitslice_mut(&mut self) -> &mut BitSlice;

    fn set_ext(&mut self, x: u32, y: u32, z: u32);
    fn index(&self, x: u32, y: u32, z: u32) -> usize;

    fn bounds(&self, axis: Axis) -> RangeInclusive<u32>;
}

/// Slice of a voxel set.
#[repr(transparent)]
pub struct VoxelSetSlice<'s> {
    inner: dyn AsVoxelSet + Send + Sync + 's,
}

impl Debug for VoxelSetSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxelSetSlice")
            .field("inner_slice", &self.inner.bitslice())
            .field("props", &self.inner.props())
            .finish()
    }
}

impl<'s> VoxelSetSlice<'s> {
    /// Whether this set contains a voxel at the given position.
    #[inline]
    pub fn contains(&self, x: u32, y: u32, z: u32) -> bool {
        self.inner
            .bitslice()
            .get(self.inner.index(x, y, z))
            .as_deref()
            .copied()
            .unwrap_or_default()
    }

    /// Sets the voxel at given position.
    #[inline]
    pub fn set(&mut self, x: u32, y: u32, z: u32) {
        let index = self.inner.index(x, y, z);
        self.inner.bitslice_mut().set(index, true);
        self.inner.set_ext(x, y, z);
    }

    /// Gets the length properties of this set.
    #[inline]
    pub fn props(&self) -> Props {
        self.inner.props()
    }

    /// Gets the length bounds of the given axis.
    #[inline]
    pub fn bounds(&self, axis: Axis) -> RangeInclusive<u32> {
        self.inner.bounds(axis)
    }

    /// Crops this set into a cropped slice.
    pub fn crop<'a>(&'a self, bounds: Bounds) -> Crop<'a, 's> {
        Crop {
            props: Props {
                len_x: *bounds.x.end() - *bounds.x.start(),
                len_y: *bounds.y.end() - *bounds.y.start(),
                len_z: *bounds.z.end() - *bounds.z.start(),
            },
            bounds,
            parent: self,
        }
    }

    /// Crops this set into a mutable cropped slice.
    pub fn crop_mut<'a>(&'a mut self, bounds: Bounds) -> CropMut<'a, 's> {
        CropMut {
            props: Props {
                len_x: *bounds.x.end() - *bounds.x.start(),
                len_y: *bounds.y.end() - *bounds.y.start(),
                len_z: *bounds.z.end() - *bounds.z.start(),
            },
            bounds,
            parent: self,
        }
    }
}

#[allow(unsafe_code)] // SAFETY: safe because the type is marked as `repr(transparent)`
impl<'a> VoxelSetSlice<'a> {
    #[inline]
    fn from_ref<'s>(this: &'s (dyn AsVoxelSet + Send + Sync + 'a)) -> &'s Self
    where
        'a: 's,
    {
        unsafe { std::mem::transmute(this) }
    }

    #[inline]
    fn from_mut<'s>(this: &'s mut (dyn AsVoxelSet + Send + Sync + 'a)) -> &'s mut Self
    where
        'a: 's,
    {
        unsafe { std::mem::transmute(this) }
    }
}

/// Basic properties of a voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Props {
    /// Length of the set in the X direction.
    pub len_x: u32,
    /// Length of the set in the Y direction.
    pub len_y: u32,
    /// Length of the set in the Z direction.
    pub len_z: u32,
}

/// Bounds of a voxel set.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bounds {
    /// The range of X values that are set.
    pub x: RangeInclusive<u32>,
    /// The range of Y values that are set.
    pub y: RangeInclusive<u32>,
    /// The range of Z values that are set.
    pub z: RangeInclusive<u32>,
}

/// A voxel set implemented using a bit set.
#[derive(Debug)]
pub struct BoxedVoxelSet {
    props: Props,
    bounds: Bounds,
    data: BitBox,
}

impl BoxedVoxelSet {
    /// Creates a new `BitSetImpl`.
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
                x: len_x..=0,
                y: len_y..=0,
                z: len_z..=0,
            },
            data: bitbox![0; len],
        }
    }

    /// Creates a new `BitSetImpl` with the given bounds.
    pub fn with_bounds(props: Props, bounds: Bounds) -> Self {
        let mut this = Self::new(props);
        this.bounds = bounds;
        this
    }
}

impl AsVoxelSet for BoxedVoxelSet {
    #[inline]
    fn props(&self) -> Props {
        self.props
    }

    #[inline]
    fn bitslice(&self) -> &BitSlice {
        &self.data
    }

    #[inline]
    fn bitslice_mut(&mut self) -> &mut BitSlice {
        &mut self.data
    }

    fn set_ext(&mut self, x: u32, y: u32, z: u32) {
        macro_rules! se {
            ($($a:ident),*$(,)?) => {
                $(self.bounds.$a =
                    *self.bounds.$a.start().min(&$a)..=*self.bounds.$a.end().max(&($a + 1));)*
            };
        }
        se![x, y, z];
    }

    #[inline]
    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        ((x * self.props.len_y + y) * self.props.len_z + z) as usize
    }

    fn bounds(&self, axis: Axis) -> RangeInclusive<u32> {
        axis.choose(
            *self.bounds.x.start(),
            *self.bounds.y.start(),
            *self.bounds.z.start(),
        )
            ..=axis.choose(
                *self.bounds.x.end(),
                *self.bounds.y.end(),
                *self.bounds.z.end(),
            )
    }
}

impl Deref for BoxedVoxelSet {
    type Target = VoxelSetSlice<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelSetSlice::from_ref(self)
    }
}

impl DerefMut for BoxedVoxelSet {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        VoxelSetSlice::from_mut(self)
    }
}

/// A cropped voxel set.
#[derive(Debug)]
pub struct Crop<'a, 's> {
    props: Props,
    bounds: Bounds,
    parent: &'a VoxelSetSlice<'s>,
}

impl AsVoxelSet for Crop<'_, '_> {
    #[inline]
    fn props(&self) -> Props {
        self.props
    }

    #[inline]
    fn bitslice(&self) -> &BitSlice {
        self.parent.inner.bitslice()
    }

    fn bitslice_mut(&mut self) -> &mut BitSlice {
        unreachable!("Cropped is only for immutable")
    }

    fn set_ext(&mut self, _x: u32, _y: u32, _z: u32) {
        unreachable!("Cropped is only for immutable")
    }

    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        self.parent.inner.index(
            *self.bounds.x.start() + x,
            *self.bounds.y.start() + y,
            *self.bounds.z.start() + z,
        )
    }

    fn bounds(&self, axis: Axis) -> RangeInclusive<u32> {
        let i = axis.choose(
            *self.bounds.x.start(),
            *self.bounds.y.start(),
            *self.bounds.z.start(),
        );
        let j = axis.choose(
            *self.bounds.x.end(),
            *self.bounds.y.end(),
            *self.bounds.z.end(),
        );
        let bounds = self.parent.inner.bounds(axis);
        (*bounds.start()).clamp(i, j) - i..=(*bounds.end()).clamp(i, j) - i
    }
}

/// A mutable cropped voxel set.
#[derive(Debug)]
pub struct CropMut<'a, 's> {
    props: Props,
    bounds: Bounds,
    parent: &'a mut VoxelSetSlice<'s>,
}

impl AsVoxelSet for CropMut<'_, '_> {
    #[inline]
    fn props(&self) -> Props {
        self.props
    }

    #[inline]
    fn bitslice(&self) -> &BitSlice {
        self.parent.inner.bitslice()
    }

    fn bitslice_mut(&mut self) -> &mut BitSlice {
        self.parent.inner.bitslice_mut()
    }

    fn set_ext(&mut self, x: u32, y: u32, z: u32) {
        self.parent.inner.set_ext(
            *self.bounds.x.start() + x,
            *self.bounds.y.start() + y,
            *self.bounds.z.start() + z,
        )
    }

    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        self.parent.inner.index(
            *self.bounds.x.start() + x,
            *self.bounds.y.start() + y,
            *self.bounds.z.start() + z,
        )
    }

    fn bounds(&self, axis: Axis) -> RangeInclusive<u32> {
        let i = axis.choose(
            *self.bounds.x.start(),
            *self.bounds.y.start(),
            *self.bounds.z.start(),
        );
        let j = axis.choose(
            *self.bounds.x.end(),
            *self.bounds.y.end(),
            *self.bounds.z.end(),
        );
        let bounds = self.parent.inner.bounds(axis);
        (*bounds.start()).clamp(i, j) - i..=(*bounds.end()).clamp(i, j) - i
    }
}

impl<'a> Deref for Crop<'a, '_> {
    type Target = VoxelSetSlice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelSetSlice::from_ref(self)
    }
}

impl<'a> Deref for CropMut<'a, '_> {
    type Target = VoxelSetSlice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VoxelSetSlice::from_ref(self)
    }
}

impl DerefMut for CropMut<'_, '_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        VoxelSetSlice::from_mut(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxed() {
        let mut set = BoxedVoxelSet::new(Props {
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
        let mut set = BoxedVoxelSet::new(Props {
            len_x: 16,
            len_y: 16,
            len_z: 16,
        });
        set.set(8, 8, 8);

        let mut cropped = set.crop_mut(Bounds {
            x: 4..=12,
            y: 4..=12,
            z: 4..=12,
        });
        assert!(cropped.contains(4, 4, 4));
        cropped.set(1, 3, 5);

        assert!(set.contains(5, 7, 9));
    }
}
