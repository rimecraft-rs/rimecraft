use std::ops::Deref;

/// A box with double-valued coords.
/// The box is axis-aligned and the coords are minimum inclusive and maximum exclusive.
#[derive(Clone, Copy, PartialEq)]
pub struct Box {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Box {
    /// Creates a box of the given positions (in (x, y, z)) as corners.
    pub fn new<T: Into<(f64, f64, f64)>>(pos1: T, pos2: T) -> Self {
        let p1 = pos1.into();
        let p2 = pos2.into();

        Self {
            min_x: if p1.0 > p2.0 { p2.0 } else { p1.0 },
            min_y: if p1.1 > p2.1 { p2.1 } else { p1.1 },
            min_z: if p1.2 > p2.2 { p2.2 } else { p1.2 },
            max_x: if p1.0 < p2.0 { p2.0 } else { p1.0 },
            max_y: if p1.1 < p2.1 { p2.1 } else { p1.1 },
            max_z: if p1.2 < p2.2 { p2.2 } else { p1.2 },
        }
    }

    pub fn shrink(mut self, x: f64, y: f64, z: f64) -> Self {
        if x < 0.0 {
            self.min_x -= x;
        } else if x > 0.0 {
            self.max_x -= x;
        }

        if y < 0.0 {
            self.min_y -= y;
        } else if x > 0.0 {
            self.max_y -= y;
        }

        if z < 0.0 {
            self.min_z -= z;
        } else if x > 0.0 {
            self.max_z -= z;
        }

        self
    }

    pub fn stretch(mut self, x: f64, y: f64, z: f64) -> Self {
        if x < 0.0 {
            self.min_x += x;
        } else if x > 0.0 {
            self.max_x += x;
        }

        if y < 0.0 {
            self.min_y += y;
        } else if x > 0.0 {
            self.max_y += y;
        }

        if z < 0.0 {
            self.min_z += z;
        } else if x > 0.0 {
            self.max_z += z;
        }

        self
    }

    pub fn expand(mut self, x: f64, y: f64, z: f64) -> Self {
        self.min_x -= x;
        self.min_y -= y;
        self.min_z -= z;
        self.max_x += x;
        self.max_y += y;
        self.max_z += z;
        self
    }

    pub fn expand_all(self, value: f64) -> Self {
        self.expand(value, value, value)
    }

    /// Creates the maximum box that this box and the given box contain.
    pub fn intersection(mut self, other: Self) -> Self {
        if self.min_x < other.min_x {
            self.min_x = other.min_x
        }

        if self.min_y < other.min_y {
            self.min_y = other.min_y
        }

        if self.min_z < other.min_z {
            self.min_z = other.min_z
        }

        if self.max_x > other.max_x {
            self.max_x = other.max_x
        }

        if self.max_y > other.max_y {
            self.max_y = other.max_y
        }

        if self.max_z > other.max_z {
            self.max_z = other.max_z
        }

        self
    }

    /// Creates the minimum box that contains this box and the given box.
    pub fn union(mut self, other: Self) -> Self {
        if self.min_x > other.min_x {
            self.min_x = other.min_x
        }

        if self.min_y > other.min_y {
            self.min_y = other.min_y
        }

        if self.min_z > other.min_z {
            self.min_z = other.min_z
        }

        if self.max_x < other.max_x {
            self.max_x = other.max_x
        }

        if self.max_y < other.max_y {
            self.max_y = other.max_y
        }

        if self.max_z < other.max_z {
            self.max_z = other.max_z
        }

        self
    }

    /// Creates a box that is translated by the given offset
    /// on each axis from this box.
    pub fn offset(mut self, x: f64, y: f64, z: f64) -> Self {
        self.min_x += x;
        self.min_y += y;
        self.min_z += z;
        self.max_x += x;
        self.max_y += y;
        self.max_z += z;
        self
    }

    pub fn is_nan(self) -> bool {
        self.min_x.is_nan()
            || self.min_y.is_nan()
            || self.min_z.is_nan()
            || self.max_x.is_nan()
            || self.max_y.is_nan()
            || self.max_z.is_nan()
    }
}

impl std::hash::Hash for Box {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.min_x.to_bits());
        state.write_u64(self.min_y.to_bits());
        state.write_u64(self.min_z.to_bits());
        state.write_u64(self.max_x.to_bits());
        state.write_u64(self.max_y.to_bits());
        state.write_u64(self.max_z.to_bits());
    }
}

impl Eq for Box {}

impl From<glam::DVec3> for Box {
    fn from(value: glam::DVec3) -> Self {
        Self {
            min_x: value.x,
            min_y: value.y,
            min_z: value.z,
            max_x: value.x + 1.0,
            max_y: value.y + 1.0,
            max_z: value.z + 1.0,
        }
    }
}

impl std::fmt::Display for Box {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Box[")?;

        self.min_x.fmt(f)?;
        f.write_str(", ")?;
        self.min_y.fmt(f)?;
        f.write_str(", ")?;
        self.min_z.fmt(f)?;

        f.write_str("] -> [")?;

        self.max_x.fmt(f)?;
        f.write_str(", ")?;
        self.max_y.fmt(f)?;
        f.write_str(", ")?;
        self.max_z.fmt(f)?;

        f.write_str("]")
    }
}

/// Represents the position of a block in a three-dimensional volume.
///
/// The position is integer-valued.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos(glam::IVec3);

impl BlockPos {
    pub const ORIGIN: Self = Self(glam::IVec3::ZERO);

    const SIZE_BITS_Z: i32 =
        1 + impl_helper::floor_log_2(impl_helper::smallest_encompassing_power_of_two(30000000));
    const SIZE_BITS_X: i32 = Self::SIZE_BITS_Z;
    const SIZE_BITS_Y: i32 = 64 - Self::SIZE_BITS_X - Self::SIZE_BITS_Z;

    // mysterious unused const
    // const BITS_X: i64 = (1 << Self::SIZE_BITS_X) - 1;

    const BITS_Y: i64 = (1 << Self::SIZE_BITS_Y) - 1;
    const BITS_Z: i64 = (1 << Self::SIZE_BITS_Z) - 1;
    const BIT_SHIFT_Z: i32 = Self::SIZE_BITS_Y;
    const BIT_SHIFT_X: i32 = Self::SIZE_BITS_Y + Self::SIZE_BITS_Z;

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(glam::IVec3 { x, y, z })
    }
}

impl Deref for BlockPos {
    type Target = glam::IVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<glam::IVec3> for BlockPos {
    fn from(value: glam::IVec3) -> Self {
        Self(value)
    }
}

impl Into<glam::IVec3> for BlockPos {
    fn into(self) -> glam::IVec3 {
        self.0
    }
}

impl Into<i64> for BlockPos {
    fn into(self) -> i64 {
        let mut l = 0_i64;
        l |= (self.x as i64 & Self::BITS_Z) << Self::BIT_SHIFT_Z;
        l |= (self.y as i64 & Self::BITS_Y) << 0;
        l | (self.z as i64 & Self::BITS_Z) << Self::BIT_SHIFT_Z
    }
}

impl From<i64> for BlockPos {
    fn from(value: i64) -> Self {
        Self(glam::IVec3 {
            x: (value << 64 - Self::BIT_SHIFT_X - Self::SIZE_BITS_X >> 64 - Self::SIZE_BITS_X)
                as i32,
            y: (value << 64 - Self::SIZE_BITS_Y >> 64 - Self::SIZE_BITS_Y) as i32,
            z: (value << 64 - Self::BIT_SHIFT_Z - Self::SIZE_BITS_Z >> 64 - Self::SIZE_BITS_Z)
                as i32,
        })
    }
}

/// An immutable pair of two integers representing
/// the X and Z coords of a chunk.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    x: i32,
    z: i32,
}

impl From<i64> for ChunkPos {
    fn from(value: i64) -> Self {
        Self {
            x: value as i32,
            z: (value >> 32) as i32,
        }
    }
}

impl Into<i64> for ChunkPos {
    fn into(self) -> i64 {
        self.x as i64 & 0xFFFFFFFF | (self.z as i64 & 0xFFFFFFFF) << 32
    }
}

/// Some translations from Minecraft: Java Edition to Rust.
pub(crate) mod impl_helper {
    const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
        0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7,
        26, 12, 18, 6, 11, 5, 10, 9,
    ];

    pub const fn is_power_of_two(value: i32) -> bool {
        value != 0 && (value & value - 1) == 0
    }

    pub const fn ceil_log_2(value: i32) -> i32 {
        let v = if is_power_of_two(value) {
            value
        } else {
            smallest_encompassing_power_of_two(value)
        };

        MULTIPLY_DE_BRUIJN_BIT_POSITION[(((v as i64) * 125613361 >> 27) & 0x1F) as usize]
    }

    pub const fn floor_log_2(value: i32) -> i32 {
        ceil_log_2(value) - if is_power_of_two(value) { 0 } else { 1 }
    }

    pub const fn smallest_encompassing_power_of_two(value: i32) -> i32 {
        let mut i = value - 1;
        i |= i >> 1;
        i |= i >> 2;
        i |= i >> 4;
        i |= i >> 8;
        i |= i >> 16;
        i + 1
    }
}
