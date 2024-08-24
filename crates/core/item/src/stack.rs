//! Item stack related types and traits.

use component::map::ComponentMap;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Reg};

use std::{fmt::Debug, marker::PhantomData};

use crate::{Item, ItemSettings, ProvideSettingsTy, RawItem};

/// Global context used for item stacks.
pub trait ItemStackCx: ProvideIdTy + ProvideSettingsTy {}

impl<T> ItemStackCx for T where T: ProvideIdTy + ProvideSettingsTy {}

/// A stack of items.
///
/// This is a data container that holds the item count and the stack's NBT.
pub struct ItemStack<'r, Cx>
where
    Cx: ItemStackCx,
{
    item: Item<'r, Cx>,
    count: u32,

    components: ComponentMap<'r, Cx>,
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ItemStackCx,
{
    /// Creates a new item stack with the given item and count.
    #[inline]
    pub fn new(item: Item<'r, Cx>, count: u32) -> Self {
        Self::with_component(
            item,
            count,
            ComponentMap::new(Reg::into_inner(item).settings().components()),
        )
    }

    /// Creates a new item stack with the given item, count, and custom NBT tag.
    pub fn with_component(
        item: Item<'r, Cx>,
        count: u32,
        components: ComponentMap<'r, Cx>,
    ) -> Self {
        Self {
            item,
            count,
            components,
        }
    }
}

impl<'r, Cx> ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>>,
{
    /// Creates an empty item stack.
    #[inline]
    pub fn empty() -> Self {
        Self::new(Item::default(), 0)
    }

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

    /// Returns the components of the stack.
    #[inline]
    pub fn components(&self) -> &ComponentMap<'r, Cx> {
        &self.components
    }

    /// Returns a mutable reference to the components of the stack.
    #[inline]
    pub fn components_mut(&mut self) -> &mut ComponentMap<'r, Cx> {
        &mut self.components
    }

    /// Sets the count of the stack.
    #[inline]
    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }
}

impl<'r, Cx> Default for ItemStack<'r, Cx>
where
    Cx: ItemStackCx + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>> + 'r,
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
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.count == other.count && self.components == other.components
    }
}

impl<Cx> Eq for ItemStack<'_, Cx> where Cx: ItemStackCx {}

impl<Cx> Clone for ItemStack<'_, Cx>
where
    Cx: ItemStackCx,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::with_component(self.item, self.count, self.components.clone())
    }
}

impl<'r, Cx> Debug for ItemStack<'r, Cx>
where
    Cx: ItemStackCx<Id: Debug, Settings<'r>: Debug> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemStack")
            .field("item", &self.item)
            .field("count", &self.count)
            .field("components", &self.components)
            .finish()
    }
}

#[cfg(feature = "serde")]
#[allow(clippy::missing_errors_doc)]
mod _serde {
    use std::{hash::Hash, str::FromStr};

    use component::{changes::ComponentChanges, RawErasedComponentType};
    use rimecraft_registry::entry::RefEntry;
    use serde::{Deserialize, Serialize};

    use super::*;

    impl<Cx> Serialize for ItemStack<'_, Cx>
    where
        Cx: ItemStackCx<Id: Serialize>,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct;

            let mut state = serializer
                .serialize_struct("ItemStack", 2 + self.components.is_empty() as usize)?;
            let entry: &RefEntry<_, _> = self.item.into();
            state.serialize_field("id", entry)?;
            state.serialize_field("count", &self.count)?;
            state.serialize_field(
                "components",
                &self
                    .components
                    .changes()
                    .ok_or_else(|| serde::ser::Error::custom("components not patched"))?,
            )?;
            state.end()
        }
    }

    impl<'r, 'de, Cx> Deserialize<'de> for ItemStack<'r, Cx>
    where
        Cx: ItemStackCx
            + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>>
            + ProvideRegistry<'r, Cx::Id, RawErasedComponentType<'r, Cx>>,
        Cx::Id: Deserialize<'de> + FromStr + Hash + Eq,
    {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<'r, Cx> {
                _marker: PhantomData<fn(&'r Cx)>,
            }

            impl<'r, 'de, Cx> serde::de::Visitor<'de> for Visitor<'r, Cx>
            where
                Cx: ItemStackCx
                    + ProvideRegistry<'r, Cx::Id, RawItem<'r, Cx>>
                    + ProvideRegistry<'r, Cx::Id, RawErasedComponentType<'r, Cx>>,
                Cx::Id: Deserialize<'de> + FromStr + Hash + Eq,
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
                    let mut components: Option<ComponentChanges<'r, 'r, Cx>> = None;

                    enum Field {
                        Id,
                        Count,
                        Tag,
                    }

                    impl<'de> Deserialize<'de> for Field {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor<'_> for FieldVisitor {
                                type Value = Field;

                                fn expecting(
                                    &self,
                                    formatter: &mut std::fmt::Formatter<'_>,
                                ) -> std::fmt::Result {
                                    formatter.write_str("`id`, `Count`, or `tag`")
                                }

                                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                                where
                                    E: serde::de::Error,
                                {
                                    match value {
                                        "id" => Ok(Field::Id),
                                        "count" => Ok(Field::Count),
                                        "components" => Ok(Field::Tag),
                                        _ => Err(serde::de::Error::unknown_field(
                                            value,
                                            &["id", "count", "components"],
                                        )),
                                    }
                                }
                            }
                            deserializer.deserialize_identifier(FieldVisitor)
                        }
                    }

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Id => {
                                if id.is_some() {
                                    return Err(serde::de::Error::duplicate_field("id"));
                                }
                                let entry: &RefEntry<Cx::Id, RawItem<'r, Cx>> = map.next_value()?;
                                id = Some(Cx::registry().of_raw(entry.raw_id()).unwrap());
                            }
                            Field::Count => {
                                count = map.next_value::<u32>()?;
                            }
                            Field::Tag => {
                                if components.is_some() {
                                    return Err(serde::de::Error::duplicate_field("components"));
                                }
                                components = Some(map.next_value()?);
                            }
                        }
                    }

                    let item = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                    Ok(ItemStack {
                        item,
                        count,
                        components: ComponentMap::with_changes(
                            Reg::into_inner(item).settings().components(),
                            components
                                .ok_or_else(|| serde::de::Error::missing_field("components"))?,
                        ),
                    })
                }
            }

            deserializer.deserialize_struct(
                "ItemStack",
                &["id", "count", "components"],
                Visitor {
                    _marker: PhantomData,
                },
            )
        }
    }
}
