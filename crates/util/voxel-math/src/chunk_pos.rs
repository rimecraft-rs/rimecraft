use crate::{BlockPos, coord_section_from_block};

/// A pair of two integers representing the X and Z coordinates of a chunk.
///
/// Chunk positions are usually serialized as an [`u64`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ChunkPos {
    /// The X coordinate of the chunk.
    pub x: i32,
    /// The Z coordinate of the chunk.
    pub z: i32,
}

impl ChunkPos {
    /// The origin chunk position.
    pub const ORIGIN: Self = Self { x: 0, z: 0 };

    /// Creates a new `ChunkPos` with the given X and Z coordinates.
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Creates a new `ChunkPos` from the given region coordinates.
    #[inline]
    pub const fn from_region(x: i32, z: i32) -> Self {
        Self::new(x << 5, z << 5)
    }

    /// Creates a new `ChunkPos` from the given region center coordinates.
    #[inline]
    pub const fn from_region_center(x: i32, z: i32) -> Self {
        Self::new((x << 5) + 31, (z << 5) + 31)
    }

    /// Returns the x-coordinate of the position.
    #[inline]
    pub const fn x(&self) -> i32 {
        self.x
    }

    /// Returns the z-coordinate of the position.
    #[inline]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

impl From<(i32, i32)> for ChunkPos {
    #[inline]
    fn from((x, z): (i32, i32)) -> Self {
        Self::new(x, z)
    }
}

impl From<ChunkPos> for u64 {
    #[inline]
    fn from(ChunkPos { x, z }: ChunkPos) -> Self {
        x as u64 & 0xFFFF_FFFF_u64 | ((z as u64 & 0xFFFF_FFFF_u64) << 32)
    }
}

impl From<u64> for ChunkPos {
    #[inline]
    fn from(value: u64) -> Self {
        Self {
            x: (value & 0xFFFF_FFFF_u64) as i32,
            z: ((value >> 32u64) & 0xFFFF_FFFF_u64) as i32,
        }
    }
}

impl From<ChunkPos> for (i32, i32) {
    #[inline]
    fn from(ChunkPos { x, z }: ChunkPos) -> Self {
        (x, z)
    }
}

impl From<BlockPos> for ChunkPos {
    #[inline]
    fn from(value: BlockPos) -> Self {
        Self {
            x: coord_section_from_block(value.x()),
            z: coord_section_from_block(value.z()),
        }
    }
}

#[cfg(feature = "edcode")]
mod _edcode {
    use edcode2::{Buf, BufMut, Decode, Encode};

    use crate::ChunkPos;

    impl<B> Encode<B> for ChunkPos
    where
        B: BufMut,
    {
        #[inline]
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            buf.put_u64((*self).into());
            Ok(())
        }
    }

    impl<'de, B> Decode<'de, B> for ChunkPos
    where
        B: Buf,
    {
        #[inline]
        fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            Ok(buf.get_u64().into())
        }
    }
}
