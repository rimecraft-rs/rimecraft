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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "Cx: ProvideIdTy, Cx::Identifier: serde::Serialize + std::hash::Hash + Eq",
        deserialize = r#"
            'r: 'de,
            Cx: ProvideIdTy,
            Cx::Identifier: serde::Deserialize<'de> + rimecraft_serde_update::Update<'de> + std::hash::Hash + Eq + 'r,
            Cx: InitAttachments + rimecraft_registry::ProvideRegistry<'r, Cx::Identifier, crate::RawItem<Cx>> + 'r"#
    ))
)]
pub struct ItemStack<'r, Cx>
where
    Cx: ProvideIdTy,
{
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde(rename = "id"),
        serde(with = "serde_helper::item_serde")
    )]
    item: Item<'r, Cx>,

    #[cfg_attr(feature = "serde", serde(rename = "Count"))]
    count: u32,

    /// Item stack's custom NBT.
    #[cfg_attr(feature = "serde", serde(rename = "tag"), serde(default))]
    nbt: Option<Compound>,

    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "serde_helper::should_skip_attachment_ser"),
        serde(default = "serde_helper::default_attachments"),
        serde(serialize_with = "serde_helper::ser_attachments"),
        serde(deserialize_with = "serde_helper::deser_attachments")
    )]
    attachments: (Attachments<Cx::Identifier>, PhantomData<Cx>),
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
    Cx: InitAttachments + ProvideRegistry<'r, Cx::Identifier, RawItem<Cx>> + 'r,
{
    /// Creates an empty item stack.
    #[inline]
    pub fn empty() -> Self {
        Self::new(Item::default(), 0)
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ProvideIdTy + ProvideRegistry<'r, Cx::Identifier, RawItem<Cx>> + 'r,
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
    pub fn attachments(&self) -> &Attachments<Cx::Identifier> {
        &self.attachments.0
    }

    /// Returns the mutable view of attachments of the stack.
    #[inline]
    pub fn attachments_mut(&mut self) -> &mut Attachments<Cx::Identifier> {
        &mut self.attachments.0
    }
}

impl<'r, Cx> Default for ItemStack<'r, Cx>
where
    Cx: InitAttachments + ProvideRegistry<'r, Cx::Identifier, RawItem<Cx>> + 'r,
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
    Cx::Identifier: Debug,
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
    fn init_attachments(attachments: &mut Attachments<Self::Identifier>);
}

#[cfg(feature = "serde")]
mod serde_helper {
    use super::*;

    use std::hash::Hash;

    #[inline]
    pub fn default_attachments<Cx>() -> (Attachments<Cx::Identifier>, PhantomData<Cx>)
    where
        Cx: InitAttachments,
    {
        let mut att = Attachments::new();
        Cx::init_attachments(&mut att);
        (att, PhantomData)
    }

    #[inline]
    pub fn should_skip_attachment_ser<K, Cx>(
        attachments: &(Attachments<K>, PhantomData<Cx>),
    ) -> bool {
        attachments.0.is_persistent_data_empty()
    }

    #[inline]
    pub fn ser_attachments<K, Cx, S>(
        attachments: &(Attachments<K>, PhantomData<Cx>),
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        K: serde::Serialize + Hash + Eq,
    {
        serde::Serialize::serialize(&attachments.0, serializer)
    }

    pub fn deser_attachments<'de, Cx, D>(
        deserializer: D,
    ) -> Result<
        (Attachments<Cx::Identifier>, PhantomData<Cx>),
        <D as serde::Deserializer<'de>>::Error,
    >
    where
        D: serde::Deserializer<'de>,
        Cx: InitAttachments,
        Cx::Identifier: serde::Deserialize<'de> + rimecraft_serde_update::Update<'de> + Hash + Eq,
    {
        use rimecraft_serde_update::Update;
        let mut attachments = Attachments::new();
        Cx::init_attachments(&mut attachments);
        attachments.update(deserializer)?;
        Ok((attachments, PhantomData))
    }

    pub mod item_serde {
        use rimecraft_serde_humanreadctl::HumanReadableControlled;
        use serde::{Deserialize, Serialize};

        use super::*;

        #[inline]
        pub fn serialize<K, Cx, S>(item: &Item<'_, Cx>, serializer: S) -> Result<S::Ok, S::Error>
        where
            Cx: ProvideIdTy,
            S: serde::Serializer,
            Cx::Identifier: Serialize + Hash + Eq,
        {
            item.serialize(HumanReadableControlled::new(serializer, true))
        }

        #[inline]
        pub fn deserialize<'rr, 'd, Cx, D>(deserializer: D) -> Result<Item<'rr, Cx>, D::Error>
        where
            'rr: 'd,
            D: serde::Deserializer<'d>,
            Cx: InitAttachments + ProvideRegistry<'rr, Cx::Identifier, RawItem<Cx>> + 'rr,
            Cx::Identifier: Deserialize<'d> + Hash + Eq + 'rr,
        {
            Item::deserialize(HumanReadableControlled::new(deserializer, true))
        }
    }
}
