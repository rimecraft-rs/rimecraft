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
    pub fn new(pos1: (f64, f64, f64), pos2: (f64, f64, f64)) -> Self {
        Self {
            min_x: if pos1.0 > pos2.0 { pos2.0 } else { pos1.0 },
            min_y: if pos1.1 > pos2.1 { pos2.1 } else { pos1.1 },
            min_z: if pos1.2 > pos2.2 { pos2.2 } else { pos1.2 },
            max_x: if pos1.0 < pos2.0 { pos2.0 } else { pos1.0 },
            max_y: if pos1.1 < pos2.1 { pos2.1 } else { pos1.1 },
            max_z: if pos1.2 < pos2.2 { pos2.2 } else { pos1.2 },
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
