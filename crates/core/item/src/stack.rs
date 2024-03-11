//! Item stack related types and traits.

use rimecraft_global_cx::{ProvideIdTy, ProvideNbtTy};
use rimecraft_registry::ProvideRegistry;

use std::{fmt::Debug, marker::PhantomData};

use crate::{Item, RawItem};

/// Global context used for item stacks.
pub trait ItemStackCx: ProvideIdTy + ProvideNbtTy {}

impl<T> ItemStackCx for T where T: ProvideIdTy + ProvideNbtTy {}

/// A stack of items.
///
/// This is a data container that holds the item count and the stack's NBT.
pub struct ItemStack<'r, Cx>
where
    Cx: ItemStackCx,
{
    item: Item<'r, Cx>,
    count: u32,

    /// Item stack's custom NBT.
    nbt: Option<Cx::Compound>,
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ItemStackCx,
{
    /// Creates a new item stack with the given item and count.
    #[inline]
    pub fn new(item: Item<'r, Cx>, count: u32) -> Self {
        Self::with_nbt(item, count, None)
    }

    /// Creates a new item stack with the given item, count, and custom NBT tag.
    pub fn with_nbt(item: Item<'r, Cx>, count: u32, nbt: Option<Cx::Compound>) -> Self {
        Self { item, count, nbt }
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<Cx>> + 'r,
{
    /// Creates an empty item stack.
    #[inline]
    pub fn empty() -> Self {
        Self::new(Item::default(), 0)
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<Cx>> + 'r,
{
    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.item == Item::default()
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ItemStackCx,
{
    /// Returns the item of the stack.
    #[inline]
    pub fn item(&self) -> Item<'r, Cx> {
        self.item
    }

    /// Returns the count of the stack.
    #[inline]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Returns the custom NBT of the stack.
    #[inline]
    pub fn nbt(&self) -> Option<&Cx::Compound> {
        self.nbt.as_ref()
    }

    /// Returns a mutable reference to the custom NBT of the stack.
    #[inline]
    pub fn nbt_mut(&mut self) -> Option<&mut Cx::Compound> {
        self.nbt.as_mut()
    }

    /// Sets the count of the stack.
    #[inline]
    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    /// Sets the custom NBT of the stack.
    #[inline]
    pub fn set_nbt(&mut self, nbt: Option<Cx::Compound>) {
        self.nbt = nbt;
    }
}

impl<Cx> ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
    Cx::Compound: Default,
{
    /// Returns the custom NBT of the stack, create one if it does not exist.
    #[inline]
    pub fn get_or_create_nbt(&mut self) -> &mut Cx::Compound {
        self.nbt.get_or_insert_with(Default::default)
    }
}

impl<'r, Cx> Default for ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<Cx>> + 'r,
{
    #[inline]
    fn default() -> Self {
        Self::new(Item::default(), 1)
    }
}

impl<'r, Cx> From<Item<'r, Cx>> for ItemStack<'r, Cx>
where
    Cx: ItemStackCx,
{
    #[inline]
    fn from(value: Item<'r, Cx>) -> Self {
        Self::new(value, 1)
    }
}

impl<'r, Cx> From<ItemStack<'r, Cx>> for Item<'r, Cx>
where
    Cx: ItemStackCx,
{
    #[inline]
    fn from(value: ItemStack<'r, Cx>) -> Self {
        value.item
    }
}

impl<Cx> PartialEq for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
    Cx::Compound: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.count == other.count && self.nbt == other.nbt
    }
}

impl<Cx> Eq for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
    Cx::Compound: Eq,
{
}

impl<Cx> Clone for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
    Cx::Compound: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::with_nbt(self.item, self.count, self.nbt.clone())
    }
}

impl<Cx> Debug for ItemStack<'_, Cx>
where
    Cx: ItemStackCx + Debug,
    Cx::Id: Debug,
    Cx::Compound: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemStack")
            .field("item", &self.item)
            .field("count", &self.count)
            .field("nbt", &self.nbt)
            .finish()
    }
}

#[cfg(feature = "serde")]
pub use _serde::{deserialize_vanilla, serialize_vanilla, DeserItemStack, SerItemStack};

#[cfg(feature = "serde")]
#[allow(clippy::missing_errors_doc)]
mod _serde {
    use std::hash::Hash;

    use rimecraft_registry::entry::RefEntry;
    use serde::{Deserialize, Serialize};

    use super::*;

    /// Global context behavior for serializing item stacks.
    pub trait SerItemStack<'r>: ItemStackCx {
        /// Serializes the item stack.
        fn serialize<S>(serializer: S, stack: &ItemStack<'r, Self>) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer;
    }

    /// Global context behavior for deserializing item stacks.
    pub trait DeserItemStack<'r, 'de>: ItemStackCx {
        /// Deserializes the item stack.
        fn deserialize<D>(deserializer: D) -> Result<ItemStack<'r, Self>, D::Error>
        where
            D: serde::Deserializer<'de>;
    }

    impl<'r, Cx> Serialize for ItemStack<'r, Cx>
    where
        Cx: SerItemStack<'r>,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Cx::serialize(serializer, self)
        }
    }

    impl<'r, 'de, Cx> Deserialize<'de> for ItemStack<'r, Cx>
    where
        Cx: DeserItemStack<'r, 'de>,
    {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Cx::deserialize(deserializer)
        }
    }

    /// Serializes the item stack in vanilla format.
    ///
    /// This is a helper function for implementing [`SerItemStack`] for a global context.
    pub fn serialize_vanilla<S, Cx>(
        serializer: S,
        stack: &ItemStack<'_, Cx>,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        Cx: ItemStackCx,
        Cx::Id: Serialize,
        Cx::Compound: Serialize,
    {
        use serde::ser::SerializeStruct;

        let mut state =
            serializer.serialize_struct("ItemStack", 2 + stack.nbt.is_some() as usize)?;
        let entry: &RefEntry<_, _> = stack.item.into();
        state.serialize_field("id", entry)?;
        state.serialize_field("Count", &stack.count)?;
        if let Some(nbt) = &stack.nbt {
            state.serialize_field("tag", nbt)?;
        }
        state.end()
    }

    /// Deserializes the item stack in vanilla format.
    ///
    /// This is a helper function for implementing [`DeserItemStack`] for a global context.
    pub fn deserialize_vanilla<'r, 'de, Cx, D>(
        deserializer: D,
    ) -> Result<ItemStack<'r, Cx>, D::Error>
    where
        'r: 'de,
        D: serde::Deserializer<'de>,
        Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<Cx>>,
        Cx::Id: Deserialize<'de> + Hash + Eq,
        Cx::Compound: Deserialize<'de>,
    {
        struct Visitor<'r, Cx> {
            _marker: PhantomData<fn(&'r Cx)>,
        }

        impl<'r, 'de, Cx> serde::de::Visitor<'de> for Visitor<'r, Cx>
        where
            'r: 'de,
            Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<Cx>>,
            Cx::Id: Deserialize<'de> + Hash + Eq,
            Cx::Compound: Deserialize<'de>,
        {
            type Value = ItemStack<'r, Cx>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a vanilla item stack structure")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut count = 0u32;
                let mut tag = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            let entry: &RefEntry<Cx::Id, RawItem<Cx>> = map.next_value()?;
                            id = Some(Cx::registry().of_raw(entry.raw_id()).unwrap());
                        }
                        "Count" => {
                            count = map.next_value::<i32>()? as u32;
                        }
                        "tag" => {
                            if tag.is_some() {
                                return Err(serde::de::Error::duplicate_field("tag"));
                            }
                            tag = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }

                Ok(ItemStack {
                    item: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                    count,
                    nbt: tag.ok_or_else(|| serde::de::Error::missing_field("tag"))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "ItemStack",
            &["id", "Count", "tag"],
            Visitor {
                _marker: PhantomData,
            },
        )
    }
}
