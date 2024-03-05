use std::hash::Hash;

use rimecraft_voxel_math::BlockPos;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// A tick of an in-game object.
#[derive(Debug, Clone, Copy)]
pub struct Tick<T> {
    ty: T,
    pos: BlockPos,
    delay: u32,
    priority: Priority,
}

impl<T> Tick<T> {
    /// Creates a new tick.
    #[inline]
    pub fn new(ty: T, pos: BlockPos) -> Self {
        Self {
            ty,
            pos,
            delay: 0,
            priority: Default::default(),
        }
    }

    /// Returns the type of the tick.
    #[inline]
    pub fn ty(&self) -> &T {
        &self.ty
    }

    /// Returns the position of the tick.
    #[inline]
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    /// Returns the delay of the tick.
    #[inline]
    pub fn delay(&self) -> u32 {
        self.delay
    }

    /// Returns the priority of the tick.
    #[inline]
    pub fn priority(&self) -> Priority {
        self.priority
    }
}

impl<T> PartialEq for Tick<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty && self.pos == other.pos
    }
}

impl<T> Eq for Tick<T> where T: Eq {}

impl<T> Hash for Tick<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ty.hash(state);
        self.pos.hash(state);
    }
}

/// A priority of a [`Tick`].
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Hash,
    Default,
    Serialize_repr,
    Deserialize_repr,
)]
#[repr(i8)]
#[allow(clippy::exhaustive_enums)]
pub enum Priority {
    /// Extremely high priority.
    ExtremelyHigh = -3,
    /// Very high priority.
    VeryHigh = -2,
    /// High priority.
    High = -1,
    /// Normal priority.
    #[default]
    Normal = 0,
    /// Low priority.
    Low = 1,
    /// Very low priority.
    VeryLow = 2,
    /// Extremely low priority.
    ExtremelyLow = 3,
}

impl From<i8> for Priority {
    fn from(value: i8) -> Self {
        match value {
            ..=-3 => Self::ExtremelyHigh,
            -2 => Self::VeryHigh,
            -1 => Self::High,
            0 => Self::Normal,
            1 => Self::Low,
            2 => Self::VeryLow,
            3.. => Self::ExtremelyLow,
        }
    }
}

mod _serde {
    use serde::{Deserialize, Serialize};

    use super::*;

    impl<T> Serialize for Tick<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            #[derive(Serialize)]
            struct Serialized<'a, T> {
                i: &'a T,
                x: i32,
                y: i32,
                z: i32,
                t: u32,
                p: Priority,
            }

            Serialized {
                i: &self.ty,
                x: self.pos.x(),
                y: self.pos.y(),
                z: self.pos.z(),
                t: self.delay,
                p: self.priority,
            }
            .serialize(serializer)
        }
    }

    impl<'de, T> Deserialize<'de> for Tick<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct Deserialized<T> {
                i: T,
                x: i32,
                y: i32,
                z: i32,
                t: u32,
                p: Priority,
            }

            let Deserialized {
                i: ty,
                x,
                y,
                z,
                t: delay,
                p: priority,
            } = Deserialized::deserialize(deserializer)?;

            Ok(Self {
                ty,
                pos: BlockPos::new(x, y, z),
                delay,
                priority,
            })
        }
    }
}
