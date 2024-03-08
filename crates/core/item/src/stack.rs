//! Item stack related types and traits.

use rimecraft_attachment::Attachments;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_nbt_ext::Compound;
use rimecraft_registry::ProvideRegistry;

use std::{fmt::Debug, marker::PhantomData};

use crate::{Item, RawItem};

/// A stack of items.
///
/// This is a data container that holds the item count and the stack's NBT.
pub struct ItemStack<'r, Cx>
where
    Cx: ProvideIdTy,
{
    item: Item<'r, Cx>,
    count: u32,

    /// Item stack's custom NBT.
    nbt: Option<Compound>,
    attachments: (Attachments<Cx::Id>, PhantomData<Cx>),
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: InitAttachments,
{
    /// Creates a new item stack with the given item and count.
    #[inline]
    pub fn new(item: Item<'r, Cx>, count: u32) -> Self {
        Self::with_nbt(item, count, None)
    }

    /// Creates a new item stack with the given item, count, and custom NBT tag.
    pub fn with_nbt(item: Item<'r, Cx>, count: u32, nbt: Option<Compound>) -> Self {
        let mut attachments = Attachments::new();
        Cx::init_attachments(&mut attachments);

        Self {
            item,
            count,
            nbt,
            attachments: (attachments, PhantomData),
        }
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: InitAttachments + ProvideRegistry<'r, Cx::Id, RawItem<Cx>> + 'r,
{
    /// Creates an empty item stack.
    #[inline]
    pub fn empty() -> Self {
        Self::new(Item::default(), 0)
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ProvideIdTy + ProvideRegistry<'r, Cx::Id, RawItem<Cx>> + 'r,
{
    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.item == Item::default()
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ProvideIdTy,
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
    pub fn nbt(&self) -> Option<&Compound> {
        self.nbt.as_ref()
    }

    /// Returns a mutable reference to the custom NBT of the stack.
    #[inline]
    pub fn nbt_mut(&mut self) -> Option<&mut Compound> {
        self.nbt.as_mut()
    }

    /// Returns the custom NBT of the stack, create one if it does not exist.
    #[inline]
    pub fn get_or_create_nbt(&mut self) -> &mut Compound {
        self.nbt.get_or_insert_with(Compound::new)
    }

    /// Sets the count of the stack.
    #[inline]
    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    /// Sets the custom NBT of the stack.
    #[inline]
    pub fn set_nbt(&mut self, nbt: Option<Compound>) {
        self.nbt = nbt;
    }

    /// Returns the attachments of the stack.
    #[inline]
    pub fn attachments(&self) -> &Attachments<Cx::Id> {
        &self.attachments.0
    }

    /// Returns the mutable view of attachments of the stack.
    #[inline]
    pub fn attachments_mut(&mut self) -> &mut Attachments<Cx::Id> {
        &mut self.attachments.0
    }
}

impl<'r, Cx> Default for ItemStack<'r, Cx>
where
    Cx: InitAttachments + ProvideRegistry<'r, Cx::Id, RawItem<Cx>> + 'r,
{
    #[inline]
    fn default() -> Self {
        Self::new(Item::default(), 1)
    }
}

impl<'r, Cx> From<Item<'r, Cx>> for ItemStack<'r, Cx>
where
    Cx: InitAttachments,
{
    #[inline]
    fn from(value: Item<'r, Cx>) -> Self {
        Self::new(value, 1)
    }
}

impl<'r, Cx> From<ItemStack<'r, Cx>> for Item<'r, Cx>
where
    Cx: ProvideIdTy,
{
    #[inline]
    fn from(value: ItemStack<'r, Cx>) -> Self {
        value.item
    }
}

impl<Cx> PartialEq for ItemStack<'_, Cx>
where
    Cx: ProvideIdTy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.count == other.count && self.nbt == other.nbt
    }
}

impl<Cx> Eq for ItemStack<'_, Cx> where Cx: ProvideIdTy {}

impl<Cx> Clone for ItemStack<'_, Cx>
where
    Cx: InitAttachments,
{
    /// Clones the item stack.
    ///
    /// This will not clone the attachments of the item stack.
    #[inline]
    fn clone(&self) -> Self {
        Self::with_nbt(self.item, self.count, self.nbt.clone())
    }
}

impl<Cx> Debug for ItemStack<'_, Cx>
where
    Cx: ProvideIdTy + Debug,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemStack")
            .field("item", &self.item)
            .field("count", &self.count)
            .field("nbt", &self.nbt)
            .field("attachments", &self.attachments)
            .finish()
    }
}

/// A trait for initializing attachments of an item stack.
pub trait InitAttachments: ProvideIdTy {
    /// Initializes the attachments of the item stack.
    fn init_attachments(attachments: &mut Attachments<Self::Id>);
}

#[cfg(feature = "serde")]
pub use _serde::{DeserItemStack, SerItemStack};

#[cfg(feature = "serde")]
mod _serde {
    use serde::{Deserialize, Serialize};

    use super::*;

    /// Global context behavior for serializing item stacks.
    pub trait SerItemStack<'r>: ProvideIdTy {
        /// Serializes the item stack.
        #[allow(clippy::missing_errors_doc)]
        fn serialize<S>(serializer: S, stack: &ItemStack<'r, Self>) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer;
    }

    /// Global context behavior for deserializing item stacks.
    pub trait DeserItemStack<'r, 'de>: ProvideIdTy {
        /// Deserializes the item stack.
        #[allow(clippy::missing_errors_doc)]
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
}
