mod event;

use std::ops::Deref;

use crate::prelude::*;

pub use event::*;

/// Represents an item.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Item(usize);

impl Item {
    pub fn new() -> Self {
        Self(0)
    }

    /// Raw id of this item.
    pub fn id(&self) -> usize {
        self.0
    }
}

impl crate::registry::Registration for Item {
    fn accept(&mut self, id: usize) {
        self.0 = id
    }
}

impl serde::Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::registry::ITEM
            .get_from_raw(self.id())
            .unwrap()
            .key()
            .value()
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let id = Identifier::deserialize(deserializer)?;

        crate::registry::ITEM
            .get_from_id(&id)
            .map(|e| Self(e.0))
            .ok_or(D::Error::custom(format!("unknown item id {id}")))
    }
}

impl Default for Item {
    fn default() -> Self {
        Self(crate::registry::ITEM.default().0)
    }
}

impl AsItem for Item {
    fn as_item(&self) -> Item {
        *self
    }
}

pub trait AsItem {
    fn as_item(&self) -> Item;
}

impl AsItem for crate::registry::Holder<Item> {
    fn as_item(&self) -> Item {
        *self.deref().deref()
    }
}

/// Represents a stack of items.
/// This is a data container that holds the
/// item count and the stack's NBT.
#[derive(Default, Clone)]
pub struct ItemStack {
    /// Count of this stack.
    pub count: u8,
    item: Item,
    nbt: Option<crate::nbt::NbtCompound>,
}

impl ItemStack {
    const UNBREAKABLE_KEY: &str = "Unbreakable";
    const DAMAGE_KEY: &str = "Damage";

    pub fn new(item: &impl AsItem, count: u8) -> Self {
        Self {
            count,
            item: item.as_item(),
            nbt: None,
        }
    }

    /// Whether this item stack is empty.
    pub fn is_empty(&self) -> bool {
        self.item == Item::default() || self.count == 0
    }

    /// Take amount of items from this stack into
    /// a new cloned stack with the taken amount.
    pub fn take(&mut self, amount: u8) -> Self {
        let i = std::cmp::min(amount, self.count);
        let mut stack = self.clone();
        stack.count = i;
        self.count -= i;
        stack
    }

    /// Take all items from this stack into a new stack.
    pub fn take_all(&mut self) -> Self {
        self.take(self.count)
    }

    /// Whether the item of this stack is in the target tag key.
    pub fn is_in(&self, tag: &crate::registry::tag::TagKey<Item>) -> bool {
        crate::registry::ITEM
            .get_from_raw(self.item.id())
            .unwrap()
            .is_in(tag)
    }

    /// Get [`Item`] inside this stack.
    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn nbt(&self) -> Option<&crate::nbt::NbtCompound> {
        self.nbt.as_ref()
    }

    pub fn nbt_mut(&mut self) -> Option<&mut crate::nbt::NbtCompound> {
        self.nbt.as_mut()
    }

    pub fn get_or_init_nbt(&mut self) -> &mut crate::nbt::NbtCompound {
        if self.nbt.is_none() {
            self.set_nbt(Some(crate::nbt::NbtCompound::new()));
        }
        self.nbt.as_mut().unwrap()
    }

    pub fn set_nbt(&mut self, nbt: Option<crate::nbt::NbtCompound>) {
        self.nbt = nbt;
        if self.is_damageable() {
            self.set_damage(self.get_damage());
        }

        if let Some(nbt) = &mut self.nbt {
            EVENTS.blocking_read().post_process_nbt(self.item, nbt);
        }
    }

    pub fn max_count(&self) -> u8 {
        EVENTS.blocking_read().get_max_count(self)
    }

    pub fn is_stackable(&self) -> bool {
        self.max_count() > 1
    }

    pub fn max_damage(&self) -> u32 {
        EVENTS.blocking_read().get_max_damage(self)
    }

    pub fn is_damageable(&self) -> bool {
        if self.is_empty() || self.max_damage() == 0 {
            false
        } else {
            self.nbt
                .as_ref()
                .map(|nbt| {
                    self.max_count() > 1
                        && nbt.get(Self::UNBREAKABLE_KEY).map_or(false, |e| match e {
                            fastnbt::Value::Byte(b) => b == &0_i8,
                            _ => false,
                        })
                })
                .unwrap_or_default()
        }
    }

    pub fn is_damaged(&self) -> bool {
        self.is_damageable() && self.get_damage() > 0
    }

    pub fn get_damage(&self) -> u32 {
        self.nbt.as_ref().map_or(0, |nbt| {
            nbt.get(Self::DAMAGE_KEY).map_or(0, |e| match e {
                fastnbt::Value::Int(int) => *int as u32,
                _ => 0,
            })
        })
    }

    pub fn set_damage(&mut self, damage: u32) {
        self.get_or_init_nbt().insert(
            Self::DAMAGE_KEY.to_string(),
            fastnbt::Value::Int(damage as i32),
        );
    }
}

impl serde::Serialize for ItemStack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RawItemStack {
            id: self.item,
            count: self.count as i8,
            tag: self.nbt.clone(),
        }
        .serialize(serializer)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RawItemStack {
    id: Item,
    #[serde(rename = "Count")]
    count: i8,
    #[serde(default)]
    tag: Option<crate::nbt::NbtCompound>,
}
