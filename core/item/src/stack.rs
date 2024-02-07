//! Item stack related types and traits.

use rimecraft_attachment::Attachments;
use rimecraft_nbt_ext::Compound;

use std::{hash::Hash, marker::PhantomData};

use crate::{Item, ToItem};

/// A stack of items.
///
/// This is a data container that holds the item count and the stack's NBT.
#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "K: serde::Serialize + Hash + Eq",
        deserialize = r#"
            'r: 'de,
            K: serde::Deserialize<'de> + rimecraft_serde_update::Update<'de> + Hash + Eq + std::fmt::Debug + 'r,
            P: InitAttachments<K> + rimecraft_registry::ProvideRegistry<'r, K, crate::RawItem<P>> + 'r"#
    ))
)]
pub struct ItemStack<'r, K, P> {
    #[cfg_attr(
        feature = "serde",
        serde(rename = "id"),
        serde(with = "serde_helper::item_serde")
    )]
    item: Item<'r, K, P>,

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
    attachments: (Attachments<K>, PhantomData<P>),
}

#[cfg(feature = "serde")]
mod serde_helper {
    use super::*;

    #[inline]
    pub fn default_attachments<K, P>() -> (Attachments<K>, PhantomData<P>)
    where
        P: InitAttachments<K>,
    {
        let mut att = Attachments::new();
        P::init_attachments(&mut att);
        (att, PhantomData)
    }

    #[inline]
    pub fn should_skip_attachment_ser<K, P>(
        attachments: &(Attachments<K>, PhantomData<P>),
    ) -> bool {
        attachments.0.is_persistent_data_empty()
    }

    #[inline]
    pub fn ser_attachments<K, P, S>(
        attachments: &(Attachments<K>, PhantomData<P>),
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        K: serde::Serialize + Hash + Eq,
    {
        serde::Serialize::serialize(&attachments.0, serializer)
    }

    pub fn deser_attachments<'de, K, P, D>(
        deserializer: D,
    ) -> Result<(Attachments<K>, PhantomData<P>), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
        P: InitAttachments<K>,
        K: serde::Deserialize<'de> + rimecraft_serde_update::Update<'de> + Hash + Eq,
    {
        use rimecraft_serde_update::Update;
        let mut attachments = Attachments::new();
        P::init_attachments(&mut attachments);
        attachments.update(deserializer)?;
        Ok((attachments, PhantomData))
    }

    pub mod item_serde {
        use rimecraft_registry::ProvideRegistry;
        use rimecraft_serde_humanreadctl::HumanReadableControlled;
        use serde::{Deserialize, Serialize};

        use crate::RawItem;

        use super::*;

        #[inline]
        pub fn serialize<K, P, S>(item: &Item<'_, K, P>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
            K: Serialize + Hash + Eq,
        {
            item.serialize(HumanReadableControlled::new(serializer, true))
        }

        #[inline]
        pub fn deserialize<'rr, 'd, K, P, D>(deserializer: D) -> Result<Item<'rr, K, P>, D::Error>
        where
            'rr: 'd,
            D: serde::Deserializer<'d>,
            K: Deserialize<'d> + Hash + Eq + std::fmt::Debug + 'rr,
            P: InitAttachments<K> + ProvideRegistry<'rr, K, RawItem<P>> + 'rr,
        {
            Item::deserialize(HumanReadableControlled::new(deserializer, true))
        }
    }
}

impl<'r, K, P> ItemStack<'r, K, P>
where
    P: InitAttachments<K>,
{
    /// Creates a new item stack with the given item and count.
    #[inline]
    pub fn new(item: Item<'r, K, P>, count: u32) -> Self {
        Self::with_nbt(item, count, None)
    }

    /// Creates a new item stack with the given item, count,
    /// and custom NBT tag.
    pub fn with_nbt(item: Item<'r, K, P>, count: u32, nbt: Option<Compound>) -> Self {
        let mut attachments = Attachments::new();
        P::init_attachments(&mut attachments);

        Self {
            item,
            count,
            nbt,
            attachments: (attachments, PhantomData),
        }
    }
}

impl<'r, K, P> ItemStack<'r, K, P> {
    /// Returns the item of the stack.
    #[inline]
    pub fn item(&self) -> Item<'r, K, P> {
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
    pub fn attachments(&self) -> &Attachments<K> {
        &self.attachments.0
    }

    /// Returns the mutable view of attachments of the stack.
    #[inline]
    pub fn attachments_mut(&mut self) -> &mut Attachments<K> {
        &mut self.attachments.0
    }
}

impl<'s, 'r, K, P> ToItem<'s, 'r, K, P> for ItemStack<'r, K, P> {
    #[inline]
    fn to_item(&'s self) -> Item<'r, K, P> {
        self.item
    }
}

/// A trait for initializing attachments of an item stack.
pub trait InitAttachments<K> {
    /// Initializes the attachments of the item stack.
    fn init_attachments(attachments: &mut Attachments<K>);
}
