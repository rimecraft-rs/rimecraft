//! Item stack related types and traits.

use component::map::ComponentMap;
use local_cx::{LocalContext, ProvideLocalCxTy};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{Reg, Registry};

use std::{fmt::Debug, marker::PhantomData};

use crate::{Item, ItemSettings, ProvideSettingsTy, RawItem};

/// Global context used for item stacks.
pub trait ItemStackCx: ProvideIdTy + ProvideSettingsTy + ProvideLocalCxTy {}

impl<T> ItemStackCx for T where T: ProvideIdTy + ProvideSettingsTy + ProvideLocalCxTy {}

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
            ComponentMap::new(Reg::to_value(item).settings().components()),
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

    /// Creates an empty item stack.
    ///
    /// # Panics
    ///
    /// Panics if the registry does not have a default entry.
    #[inline]
    pub fn empty<Local>(cx: Local) -> Self
    where
        Local: LocalContext<&'r Registry<Cx::Id, RawItem<'r, Cx>>>,
    {
        Self::new(
            cx.acquire()
                .default_entry()
                .expect("default item not found in the registry"),
            0,
        )
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0 || Reg::to_entry(self.item).is_default()
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

    use component::{RawErasedComponentType, changes::ComponentChanges};
    use local_cx::{
        LocalContextExt as _, WithLocalCx,
        serde::{DeserializeWithCx, SerializeWithCx},
    };
    use rimecraft_registry::entry::RefEntry;
    use serde::{Deserialize, Serialize};

    use super::*;

    impl<'a, Cx> SerializeWithCx<Cx::LocalContext<'a>> for ItemStack<'a, Cx>
    where
        Cx: ItemStackCx<Id: Serialize>,
    {
        fn serialize_with_cx<S>(
            &self,
            serializer: WithLocalCx<S, Cx::LocalContext<'a>>,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct;

            let cx = serializer.local_cx;
            let mut state = serializer
                .inner
                .serialize_struct("ItemStack", 2 + self.components.is_empty() as usize)?;
            let entry = Item::to_entry(self.item);
            state.serialize_field("id", entry)?;
            state.serialize_field("count", &self.count)?;
            state.serialize_field(
                "components",
                &cx.with(
                    self.components
                        .changes()
                        .ok_or_else(|| serde::ser::Error::custom("components not patched"))?,
                ),
            )?;
            state.end()
        }
    }

    impl<'r, 'de, Cx> DeserializeWithCx<'de, Cx::LocalContext<'r>> for ItemStack<'r, Cx>
    where
        Cx: ItemStackCx,
        Cx::Id: Deserialize<'de> + FromStr + Hash + Eq,
        Cx::LocalContext<'r>: LocalContext<&'r Registry<Cx::Id, RawItem<'r, Cx>>>
            + LocalContext<&'r Registry<Cx::Id, RawErasedComponentType<'r, Cx>>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: WithLocalCx<D, Cx::LocalContext<'r>>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<'r, Cx, L> {
                _marker: PhantomData<fn(&'r Cx)>,
                cx: L,
            }

            impl<'r, 'de, Cx> serde::de::Visitor<'de> for Visitor<'r, Cx, Cx::LocalContext<'r>>
            where
                Cx: ItemStackCx,
                Cx::Id: Deserialize<'de> + FromStr + Hash + Eq,
                Cx::LocalContext<'r>: LocalContext<&'r Registry<Cx::Id, RawItem<'r, Cx>>>
                    + LocalContext<&'r Registry<Cx::Id, RawErasedComponentType<'r, Cx>>>,
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
                                let entry: &RefEntry<Cx::Id, RawItem<'r, Cx>> = map
                                    .next_value_seed(WithLocalCx {
                                        inner: PhantomData,
                                        local_cx: self.cx,
                                    })?;
                                id = Some(
                                    std::convert::identity::<&Registry<_, RawItem<'_, _>>>(
                                        self.cx.acquire(),
                                    )
                                    .of_raw(entry.raw_id())
                                    .unwrap(),
                                );
                            }
                            Field::Count => {
                                count = map.next_value::<u32>()?;
                            }
                            Field::Tag => {
                                if components.is_some() {
                                    return Err(serde::de::Error::duplicate_field("components"));
                                }
                                components = Some(map.next_value_seed(WithLocalCx {
                                    inner: PhantomData,
                                    local_cx: self.cx,
                                })?);
                            }
                        }
                    }

                    let item = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                    Ok(ItemStack {
                        item,
                        count,
                        components: ComponentMap::with_changes(
                            Reg::to_value(item).settings().components(),
                            components
                                .ok_or_else(|| serde::de::Error::missing_field("components"))?,
                        ),
                    })
                }
            }

            let cx = deserializer.local_cx;
            deserializer.inner.deserialize_struct(
                "ItemStack",
                &["id", "count", "components"],
                Visitor {
                    _marker: PhantomData,
                    cx,
                },
            )
        }
    }
}
