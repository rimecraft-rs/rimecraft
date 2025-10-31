use glam::IVec3;

use crate::{BlockPos, ChunkPos, coord_block_from_section};

/// Position of a chunk section.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ChunkSectionPos(pub IVec3);

impl ChunkSectionPos {
    /// Creates a new `ChunkSectionPos` with the given X, Y, and Z coordinates.
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    /// Returns the x-coordinate of the position.
    #[inline]
    pub const fn x(&self) -> i32 {
        self.0.x
    }

    /// Returns the y-coordinate of the position.
    #[inline]
    pub const fn y(&self) -> i32 {
        self.0.y
    }

    /// Returns the z-coordinate of the position.
    #[inline]
    pub const fn z(&self) -> i32 {
        self.0.z
    }

    /// Returns the minimum block position of the chunk section.
    #[inline]
    pub const fn min_pos(self) -> BlockPos {
        BlockPos(IVec3 {
            x: coord_block_from_section(self.x()),
            y: coord_block_from_section(self.y()),
            z: coord_block_from_section(self.z()),
        })
    }
}

impl From<IVec3> for ChunkSectionPos {
    #[inline]
    fn from(pos: IVec3) -> Self {
        Self(pos)
    }
}

impl From<ChunkSectionPos> for IVec3 {
    #[inline]
    fn from(pos: ChunkSectionPos) -> Self {
        pos.0
    }
}

impl From<(i32, i32, i32)> for ChunkSectionPos {
    #[inline]
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<ChunkSectionPos> for (i32, i32, i32) {
    #[inline]
    fn from(pos: ChunkSectionPos) -> (i32, i32, i32) {
        (pos.x(), pos.y(), pos.z())
    }
}

impl From<(ChunkPos, i32)> for ChunkSectionPos {
    #[inline]
    fn from(value: (ChunkPos, i32)) -> Self {
        Self::new(value.0.x, value.1, value.0.z)
    }
}

impl std::ops::Add<IVec3> for ChunkSectionPos {
    type Output = Self;

    #[inline]
    fn add(self, rhs: IVec3) -> Self {
        Self(self.0 + rhs)
    }
}

impl std::ops::AddAssign<IVec3> for ChunkSectionPos {
    #[inline]
    fn add_assign(&mut self, rhs: IVec3) {
        self.0 += rhs;
    }
}

impl std::ops::Sub<IVec3> for ChunkSectionPos {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: IVec3) -> Self {
        Self(self.0 - rhs)
    }
}

impl std::ops::SubAssign<IVec3> for ChunkSectionPos {
    #[inline]
    fn sub_assign(&mut self, rhs: IVec3) {
        self.0 -= rhs;
    }
}

impl From<u64> for ChunkSectionPos {
    #[inline]
    fn from(value: u64) -> Self {
        Self(IVec3 {
            x: (value >> 42) as i32,
            y: (value << 44 >> 44) as i32,
            z: (value << 22 >> 42) as i32,
        })
    }
}

impl From<ChunkSectionPos> for u64 {
    #[inline]
    fn from(ChunkSectionPos(IVec3 { x, y, z }): ChunkSectionPos) -> Self {
        let mut l = 0u64;
        l |= (x as Self & 0x003F_FFFF) << 42;
        l |= y as Self & 0x000F_FFFF;
        l | ((z as Self & 0x003F_FFFF) << 20)
    }
}

#[cfg(feature = "edcode")]
mod _edcode {

    use edcode2::{Buf, BufMut, Decode, Encode};

    use super::*;

    impl<B> Encode<B> for ChunkSectionPos
    where
        B: BufMut,
    {
        #[inline]
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            buf.put_u64((*self).into());
            Ok(())
        }
    }

    impl<'de, B> Decode<'de, B> for ChunkSectionPos
    where
        B: Buf,
    {
        #[inline]
        fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            Ok(buf.get_u64().into())
        }
    }
}
