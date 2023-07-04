mod event;

use std::ops::Deref;

use crate::{
    prelude::*,
    registry::{Registration, RegistryAccess},
};

pub use event::*;

/// Represents an item.
#[derive(Clone, Copy)]
pub struct Item {
    id: usize,
    properties: ItemDescriptor,
}

/// Describes some basic properties of an item.
#[derive(Clone, Copy)]
pub struct ItemDescriptor {
    pub sync_nbt: bool,
}

impl Default for ItemDescriptor {
    fn default() -> Self {
        Self { sync_nbt: true }
    }
}

impl Item {
    pub fn new(descriptor: ItemDescriptor) -> Self {
        Self {
            id: 0,
            properties: descriptor,
        }
    }

    pub fn descriptor(&self) -> &ItemDescriptor {
        &self.properties
    }
}

impl Registration for Item {
    fn accept(&mut self, id: usize) {
        self.id = id
    }

    fn raw_id(&self) -> usize {
        self.id
    }
}

impl RegistryAccess for Item {
    fn registry() -> &'static crate::registry::Registry<Self> {
        crate::registry::ITEM.deref()
    }
}

impl serde::Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::registry::ITEM
            .get_from_raw(self.raw_id())
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
        let id = Identifier::deserialize(deserializer)?;
        Ok(crate::registry::ITEM.get_from_id(&id).map_or_else(
            || {
                tracing::debug!("Tried to load invalid item: {id}");
                crate::registry::ITEM.default_entry().1.as_item()
            },
            |e| *e.1.deref(),
        ))
    }
}

impl Default for Item {
    fn default() -> Self {
        *crate::registry::ITEM.default_entry().1.deref()
    }
}

impl AsItem for Item {
    fn as_item(&self) -> Item {
        *self
    }
}

/// A trait for converting into [`Item`].
pub trait AsItem {
    fn as_item(&self) -> Item;
}

impl AsItem for crate::registry::Holder<Item> {
    /// Convert this object into an item.
    fn as_item(&self) -> Item {
        *self.deref().deref()
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::registry::ITEM
            .get_from_raw(self.raw_id())
            .ok_or(std::fmt::Error)?
            .key()
            .value()
            .fmt(f)
    }
}

impl Eq for Item {}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id)
    }
}

/// Represents a stack of items.
/// This is a data container that holds the
/// item count and the stack's NBT.
#[derive(Default, Clone, PartialEq)]
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

    /// Get [`Item`] inside this stack.
    pub fn item(&self) -> Item {
        self.item
    }

    /// Whether the target item holder matches the provided predicate.
    pub fn matches<F: Fn(&crate::registry::Holder<Item>) -> bool>(&self, f: F) -> bool {
        f(crate::registry::ITEM
            .get_from_raw(self.item.raw_id())
            .unwrap())
    }

    pub fn nbt(&self) -> Option<&crate::nbt::NbtCompound> {
        self.nbt.as_ref()
    }

    pub fn nbt_mut(&mut self) -> Option<&mut crate::nbt::NbtCompound> {
        self.nbt.as_mut()
    }

    pub fn get_or_init_nbt(&mut self) -> &mut crate::nbt::NbtCompound {
        self.nbt
            .get_or_insert_with(|| crate::nbt::NbtCompound::new())
    }

    pub fn set_nbt(&mut self, nbt: Option<crate::nbt::NbtCompound>) {
        self.nbt = nbt;
        if self.is_damageable() {
            self.set_damage(self.damage());
        }

        if let Some(nbt) = &mut self.nbt {
            EVENTS.read().post_process_nbt(self.item, nbt);
        }
    }

    pub fn max_count(&self) -> u8 {
        EVENTS.read().get_max_count(self)
    }

    pub fn is_stackable(&self) -> bool {
        self.max_count() > 1
    }

    pub fn max_damage(&self) -> u32 {
        EVENTS.read().get_max_damage(self)
    }

    pub fn is_damageable(&self) -> bool {
        if self.is_empty() || self.max_damage() == 0 {
            false
        } else {
            self.nbt.as_ref().map_or(true, |nbt| {
                !nbt.get_bool(Self::UNBREAKABLE_KEY).unwrap_or_default()
            })
        }
    }

    pub fn is_damaged(&self) -> bool {
        self.is_damageable() && self.damage() > 0
    }

    /// Get damage of this satck from the nbt tags.
    pub fn damage(&self) -> u32 {
        self.nbt.as_ref().map_or(0, |nbt| {
            nbt.get_i32(Self::DAMAGE_KEY).unwrap_or_default() as u32
        })
    }

    pub fn set_damage(&mut self, damage: u32) {
        self.get_or_init_nbt()
            .insert_i32(Self::DAMAGE_KEY, damage as i32);
    }

    /// Whether the given item stack's items and NBT are equal with this stack.
    pub fn can_combine(&self, other: &Self) -> bool {
        if self.item() != other.item() {
            false
        } else if self.is_empty() && other.is_empty() {
            true
        } else {
            self.nbt == other.nbt
        }
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

impl<'de> serde::Deserialize<'de> for ItemStack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut raw = RawItemStack::deserialize(deserializer)?;
        let item = raw.id;
        if let Some(nbt) = &mut raw.tag {
            EVENTS.read().post_process_nbt(item, nbt);
        }
        let mut stack = Self {
            count: raw.count as u8,
            item: raw.id,
            nbt: raw.tag,
        };
        if stack.is_damageable() {
            stack.set_damage(stack.damage());
        }
        Ok(stack)
    }
}

impl AsItem for ItemStack {
    fn as_item(&self) -> Item {
        self.item()
    }
}

impl std::fmt::Display for ItemStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.count.fmt(f)?;
        f.write_str(" ")?;
        self.item.fmt(f)
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
