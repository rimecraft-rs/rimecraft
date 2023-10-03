use std::hash::Hash;

use rimecraft_nbt_ext::CompoundExt;

use crate::prelude::*;

pub struct Tick<T> {
    pub value: T,
    pub pos: BlockPos,
    pub delay: u32,
    pub priority: Priority,
}

impl<T> Tick<T> {
    const TYPE_NBT_KEY: &str = "i";
    const X_NBT_KEY: &str = "x";
    const Y_NBT_KEY: &str = "y";
    const Z_NBT_KEY: &str = "z";
    const DELAY_NBT_KEY: &str = "t";
    const PRIORITY_NBT_KEY: &str = "p";

    pub fn new(value: T, pos: BlockPos) -> Self {
        Self {
            value,
            pos,
            delay: 0,
            priority: Priority::Normal,
        }
    }

    pub fn tick<F, C>(
        tick_list: &[fastnbt::Value],
        name_to_type_fn: F,
        pos: crate::util::math::ChunkPos,
        tick_consumer: C,
    ) where
        F: Fn(&str) -> Option<T>,
        C: Fn(&Self),
    {
        let l: i64 = pos.into();
        for nbt in tick_list.iter() {
            let tick = if let fastnbt::Value::Compound(ref value) = nbt {
                Self::from_nbt(value, |n| name_to_type_fn(n))
            } else {
                None
            };

            if let Some(t) = tick {
                if Into::<i64>::into(t.pos) == l {
                    tick_consumer(&t);
                }
            }
        }
    }

    pub fn from_nbt<F>(nbt: &rimecraft_nbt_ext::Compound, name_to_type: F) -> Option<Self>
    where
        F: Fn(&str) -> Option<T>,
    {
        name_to_type(nbt.get_str(Self::TYPE_NBT_KEY)?)
            .map::<Option<Self>, _>(|t| {
                let pos = BlockPos::new(
                    nbt.get_i32(Self::X_NBT_KEY)?,
                    nbt.get_i32(Self::Y_NBT_KEY)?,
                    nbt.get_i32(Self::Z_NBT_KEY)?,
                );
                Some(Self {
                    value: t,
                    pos,
                    delay: nbt.get_i32(Self::DELAY_NBT_KEY)? as u32,
                    priority: Priority::by_index(nbt.get_i32(Self::PRIORITY_NBT_KEY)? as i8),
                })
            })
            .flatten()
    }

    pub fn to_nbt<F>(&self, type_to_name_fn: F) -> rimecraft_nbt_ext::Compound
    where
        F: Fn(&T) -> String,
    {
        let mut compound = rimecraft_nbt_ext::Compound::new();
        compound.insert_str(Self::TYPE_NBT_KEY, &type_to_name_fn(&self.value));

        compound.insert_i32(Self::X_NBT_KEY, self.pos.x);
        compound.insert_i32(Self::Y_NBT_KEY, self.pos.y);
        compound.insert_i32(Self::Z_NBT_KEY, self.pos.z);

        compound.insert_i32(Self::DELAY_NBT_KEY, self.delay as i32);
        compound.insert_i32(Self::PRIORITY_NBT_KEY, self.priority as i32);

        compound
    }
}

impl<T: Hash> Hash for Tick<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.pos.hash(state);
    }
}

impl<T: PartialEq> PartialEq for Tick<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.pos == other.pos
    }
}

impl<T: Eq> Eq for Tick<T> {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, enumn::N)]
pub enum Priority {
    ExtremelyHigh = -3,
    VeryHigh = -2,
    High = -1,
    Normal = 0,
    Low = 1,
    VeryLow = 2,
    ExtremelyLow = 3,
}

impl Priority {
    const VALUES: [Self; 7] = [
        Self::ExtremelyHigh,
        Self::VeryHigh,
        Self::High,
        Self::Normal,
        Self::Low,
        Self::VeryLow,
        Self::ExtremelyLow,
    ];

    pub fn by_index(index: i8) -> Self {
        Self::VALUES
            .into_iter()
            .find(|e| *e as i8 == index)
            .unwrap_or_else(|| {
                if index < -3 {
                    Self::ExtremelyHigh
                } else {
                    Self::ExtremelyLow
                }
            })
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::Normal
    }
}
