//! Position types.

use std::fmt::Display;

use glam::{DVec3, IVec3};

/// A position of a block in a three-dimensional volume.
///
/// The position is integer-valued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos(pub IVec3);

impl BlockPos {
    /// The origin of the coordinate system.
    pub const ORIGIN: Self = Self(IVec3::ZERO);

    /// Creates a new block position.
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
}

impl From<IVec3> for BlockPos {
    #[inline]
    fn from(pos: IVec3) -> Self {
        Self(pos)
    }
}

impl From<(i32, i32, i32)> for BlockPos {
    #[inline]
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<BlockPos> for IVec3 {
    #[inline]
    fn from(pos: BlockPos) -> Self {
        pos.0
    }
}

impl From<BlockPos> for DVec3 {
    #[inline]
    fn from(value: BlockPos) -> Self {
        IVec3::from(value).into()
    }
}

impl From<BlockPos> for (i32, i32, i32) {
    #[inline]
    fn from(pos: BlockPos) -> Self {
        (pos.x(), pos.y(), pos.z())
    }
}

impl std::ops::Add<IVec3> for BlockPos {
    type Output = BlockPos;

    #[inline]
    fn add(self, rhs: IVec3) -> BlockPos {
        BlockPos(self.0 + rhs)
    }
}

impl std::ops::AddAssign<IVec3> for BlockPos {
    #[inline]
    fn add_assign(&mut self, rhs: IVec3) {
        self.0 += rhs;
    }
}

impl std::ops::Sub<IVec3> for BlockPos {
    type Output = BlockPos;

    #[inline]
    fn sub(self, rhs: IVec3) -> BlockPos {
        BlockPos(self.0 - rhs)
    }
}

impl std::ops::SubAssign<IVec3> for BlockPos {
    #[inline]
    fn sub_assign(&mut self, rhs: IVec3) {
        self.0 -= rhs;
    }
}

const LEN_BITS_XZ: i32 = 1 + (1i32 << (32 - (30000000i32 - 1).leading_zeros())).ilog2() as i32;
const LEN_BITS_Y: i32 = 64 - LEN_BITS_XZ - LEN_BITS_XZ;

const BITS_X: i64 = (1i64 << LEN_BITS_XZ) - 1;
const BITS_Y: i64 = (1i64 << LEN_BITS_Y) - 1;
const BITS_Z: i64 = (1i64 << LEN_BITS_XZ) - 1;

const BIT_SHIFT_X: i32 = LEN_BITS_Y + LEN_BITS_XZ;
const BIT_SHIFT_Z: i32 = LEN_BITS_Y;

impl From<BlockPos> for i64 {
    #[inline]
    fn from(BlockPos(IVec3 { x, y, z }): BlockPos) -> i64 {
        let mut l = 0i64;
        l |= (x as i64 & BITS_X) << BIT_SHIFT_X;
        l |= y as i64 & BITS_Y;
        l | ((z as i64 & BITS_Z) << BIT_SHIFT_Z)
    }
}

impl From<i64> for BlockPos {
    #[inline]
    fn from(l: i64) -> BlockPos {
        Self(IVec3 {
            x: (l << (64 - BIT_SHIFT_X - LEN_BITS_XZ) >> (64 - LEN_BITS_XZ)) as i32,
            y: (l << (64 - LEN_BITS_Y) >> (64 - LEN_BITS_Y)) as i32,
            z: (l << (64 - BIT_SHIFT_Z - LEN_BITS_XZ) >> (64 - LEN_BITS_XZ)) as i32,
        })
    }
}

impl Display for BlockPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

#[cfg(feature = "serde")]
mod _serde {
    use serde::{Deserialize, Serialize};

    use super::*;

    impl Serialize for BlockPos {
        /// Serializes the block position as a sequence of three integers.
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            [self.0.x, self.0.y, self.0.z].serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for BlockPos {
        /// Deserializes the block position from a sequence of three integers
        /// or a struct of three dimensions.
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter
                        .write_str("a sequence of three integers, or a struct of three dimensions")
                }

                type Value = BlockPos;

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let x = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(0, &"a sequence of three integers")
                    })?;
                    let y = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(1, &"a sequence of three integers")
                    })?;
                    let z = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(2, &"a sequence of three integers")
                    })?;
                    Ok(BlockPos::new(x, y, z))
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    use ::serde::de::Error;
                    let mut x = None;
                    let mut y = None;
                    let mut z = None;
                    while let Some((k, v)) = map.next_entry::<&str, i32>()? {
                        let Some(c) = k.chars().next() else {
                            continue;
                        };
                        match c {
                            'x' | 'X' => x = Some(v),
                            'y' | 'Y' => y = Some(v),
                            'z' | 'Z' => z = Some(v),
                            _ => return Err(Error::unknown_field(k, &["x", "y", "z"])),
                        }
                    }
                    let x = x.ok_or_else(|| Error::missing_field("x"))?;
                    let y = y.ok_or_else(|| Error::missing_field("y"))?;
                    let z = z.ok_or_else(|| Error::missing_field("z"))?;
                    Ok(BlockPos::new(x, y, z))
                }
            }

            deserializer.deserialize_any(Visitor)
        }
    }
}

#[cfg(feature = "edcode")]
mod edcode {

    use edcode2::{Buf, BufMut, Decode, Encode};

    use super::*;

    impl<B> Encode<B> for BlockPos
    where
        B: BufMut,
    {
        #[inline]
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            buf.put_i64((*self).into());
            Ok(())
        }
    }

    impl<'de, B> Decode<'de, B> for BlockPos
    where
        B: Buf,
    {
        const SUPPORT_NON_IN_PLACE: bool = true;

        #[inline]
        fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            Ok(buf.get_i64().into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_conv() {
        let pos = BlockPos::new(1, 2, 3);
        let l: i64 = pos.into();
        let pos2: BlockPos = l.into();
        assert_eq!(pos, pos2);
    }
}
