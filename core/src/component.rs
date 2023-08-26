use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use tracing::{trace_span, warn};

use crate::{
    nbt::NbtElement,
    net::{Decode, Encode, NetSync},
};

/// Represents a type of component that can be attached
/// on [`Components`].
pub trait Attach {
    /// Actions to perform before attaching this component
    /// on components.
    /// By this, you can interact with other components to
    /// implement functions like syncing components.
    ///
    /// Don't attach this component in this function.
    fn on_attach(&mut self, components: &mut Components);
}

/// Manager of components.
pub struct Components {
    components: HashMap<crate::Id, (Box<dyn Attach + Send + Sync>, TypeId)>,
}

impl Components {
    /// Creates a new empty components instance,
    /// without networking and saving features.
    ///
    /// To create with external features,
    /// see [`Self::builder()`].
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Creates a new [`ComponentsBuilder`].
    #[inline]
    pub fn builder() -> ComponentsBuilder {
        ComponentsBuilder { inner: Self::new() }
    }

    /// Register a component into this instance.
    /// The component should implement [`Attach`].
    pub fn register<T>(&mut self, id: crate::Id, component: T)
    where
        T: Attach + Send + Sync + 'static,
    {
        assert!(
            !self.components.contains_key(&id),
            "component with id {id} already exist in this components!"
        );

        let mut boxed = Box::new(component);
        boxed.on_attach(self);
        self.components.insert(id, (boxed, TypeId::of::<T>()));
    }

    /// Get a static typed component from this instance.
    pub fn get<T>(&self, id: &crate::Id) -> Option<&T>
    where
        T: Attach + Send + Sync + 'static,
    {
        self.components
            .get(id)
            .map(|value| {
                if value.1 == TypeId::of::<T>() {
                    Some(unsafe {
                        &*(value.0.deref() as *const (dyn Attach + Send + Sync) as *const T)
                    })
                } else {
                    None
                }
            })
            .flatten()
    }

    /// Get a mutable static typed component from this instance.
    pub fn get_mut<T>(&mut self, id: &crate::Id) -> Option<&mut T>
    where
        T: Attach + Send + Sync + 'static,
    {
        self.components
            .get_mut(id)
            .map(|value| {
                if value.1 == TypeId::of::<T>() {
                    Some(unsafe {
                        &mut *(value.0.deref_mut() as *mut (dyn Attach + Send + Sync) as *mut T)
                    })
                } else {
                    None
                }
            })
            .flatten()
    }
}

impl Encode for Components {
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        let Component(event) =
            self.get::<Component<
                crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>,
            >>(NET_SEND_ID.deref())
                .expect("net send event component not found");

        let mut hashmap = HashMap::new();
        event.invoker()(&mut hashmap)?;
        hashmap.encode(buf)
    }
}

impl NetSync for Components {
    fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        let Component(event) =
            self.get::<Component<
                crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>,
            >>(NET_SEND_ID.deref())
                .expect("net recv event component not found");

        let mut hashmap = HashMap::<crate::Id, Bytes>::decode(buf)?;
        event.invoker()(&mut hashmap)
    }
}

impl serde::Serialize for Components {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Component(event) = self
            .get::<Component<
                crate::Event<
                    dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt_rc::error::Result<()>,
                >,
            >>(NBT_SAVE_ID.deref())
            .expect("net send event component not found");

        let mut hashmap = HashMap::new();

        use serde::ser::Error;
        event.invoker()(&mut hashmap)
            .map_err(|err| <S as serde::Serializer>::Error::custom(err))?;
        hashmap.serialize(serializer)
    }
}

impl crate::nbt::Update for Components {
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Component(event) = self
            .get::<Component<
                crate::Event<
                    dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt_rc::error::Result<()>,
                >,
            >>(NBT_READ_ID.deref())
            .expect("net recv event component not found");

        use serde::{de::Error, Deserialize};
        let mut hashmap = HashMap::deserialize(deserializer)?;
        event.invoker()(&mut hashmap)
            .map_err(|err| <D as serde::Deserializer<'_>>::Error::custom(err))
    }
}

/// [`Components`] builder for creating with external features.
pub struct ComponentsBuilder {
    inner: Components,
}

impl ComponentsBuilder {
    /// Enable networking features for the instance.
    pub fn net_sync(mut self) -> Self {
        self.inner
            .register(net_send_event_comp_id(), net_event_comp());
        self.inner
            .register(net_recv_event_comp_id(), net_event_comp());
        self
    }

    /// Enable nbt reading and writing feature for the instance.
    pub fn nbt_storing(mut self) -> Self {
        self.inner
            .register(nbt_save_event_comp_id(), nbt_event_comp());
        self.inner
            .register(nbt_read_event_comp_id(), nbt_event_comp());
        self
    }

    /// Build this instance into [`Components`].
    #[inline]
    pub fn build(self) -> Components {
        self.inner
    }
}

impl Into<Components> for ComponentsBuilder {
    fn into(self) -> Components {
        self.build()
    }
}

/// Represents a simple component without extra
/// attach features, which has an empty
/// implementation of [`Attach`].
pub struct Component<T>(pub T);

impl<T> Attach for Component<T> {
    fn on_attach(&mut self, _components: &mut Components) {}
}

impl<T> Deref for Component<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static NET_SEND_ID: once_cell::sync::Lazy<crate::Id> =
    once_cell::sync::Lazy::new(net_send_event_comp_id);
static NET_RECV_ID: once_cell::sync::Lazy<crate::Id> =
    once_cell::sync::Lazy::new(net_recv_event_comp_id);

/// Represents a component that able to sync by
/// networking methods, through [`NetSync`] trait.
///
/// The `1` field is the component id which is used
/// to be registered into components.
pub struct Synced<T>(pub T, pub crate::Id)
where
    T: Attach + NetSync + 'static;

impl<T> Attach for Synced<T>
where
    T: Attach + NetSync + 'static,
{
    fn on_attach(&mut self, components: &mut Components) {
        self.0.on_attach(components);

        let ptr = self as *mut Self;

        let span = trace_span!(
            "attach synced component",
            comp_type = std::any::type_name::<T>()
        );

        let _ = span.enter();

        if let Some(Component(event)) = components.get_mut::<Component<
            crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>,
        >>(NET_SEND_ID.deref())
        {
            event.register(Box::new(move |map| {
                let this = unsafe { &*ptr };

                map.insert(this.1.clone(), {
                    let mut bytes_mut = bytes::BytesMut::new();
                    this.0.encode(&mut bytes_mut)?;

                    bytes_mut.into()
                });

                Ok(())
            }))
        } else {
            warn!("network sending event not found");
        }

        if let Some(Component(event)) = components.get_mut::<Component<
            crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>,
        >>(NET_RECV_ID.deref())
        {
            event.register(Box::new(move |map| {
                let this = unsafe { &mut *ptr };
                let mut bytes = map.remove(&this.1).unwrap();

                this.0.read_buf(&mut bytes)
            }))
        } else {
            warn!("network receiving event not found");
        }
    }
}

impl<T> Deref for Synced<T>
where
    T: Attach + NetSync + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Synced<T>
where
    T: Attach + NetSync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn net_event_comp(
) -> Component<crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>> {
    Component(crate::Event::new(|listeners| {
        Box::new(move |map| {
            for listener in listeners {
                listener(map)?;
            }

            Ok(())
        })
    }))
}

#[inline]
pub fn net_send_event_comp_id() -> crate::Id {
    crate::Id::new("core", "net_send".to_string())
}

#[inline]
pub fn net_recv_event_comp_id() -> crate::Id {
    crate::Id::new("core", "net_recv".to_string())
}

static NBT_SAVE_ID: once_cell::sync::Lazy<crate::Id> =
    once_cell::sync::Lazy::new(nbt_save_event_comp_id);
static NBT_READ_ID: once_cell::sync::Lazy<crate::Id> =
    once_cell::sync::Lazy::new(nbt_read_event_comp_id);

/// Represents a component that able to be stored
/// by nbt, through [`crate::nbt::Update`] trait.
///
/// The `1` field is the component id which is used
/// to be registered into components.
pub struct Saved<T>(pub T, pub crate::Id)
where
    T: Attach + crate::nbt::Update + 'static;

impl<T> Attach for Saved<T>
where
    T: Attach + crate::nbt::Update + 'static,
{
    fn on_attach(&mut self, components: &mut Components) {
        self.0.on_attach(components);

        let ptr = self as *mut Self;

        let span = trace_span!(
            "attach saved component",
            comp_type = std::any::type_name::<T>()
        );

        let _ = span.enter();

        if let Some(Component(event)) = components.get_mut::<Component<
            crate::Event<
                dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt_rc::error::Result<()>,
            >,
        >>(NBT_SAVE_ID.deref())
        {
            event.register(Box::new(move |map| {
                let this = unsafe { &*ptr };
                map.insert(
                    this.1.clone(),
                    this.0.serialize(&mut fastnbt_rc::value::Serializer)?,
                );

                Ok(())
            }))
        } else {
            warn!("nbt saving event not found");
        }

        if let Some(Component(event)) = components.get_mut::<Component<
            crate::Event<
                dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt_rc::error::Result<()>,
            >,
        >>(NBT_READ_ID.deref())
        {
            event.register(Box::new(move |map| {
                let this = unsafe { &mut *ptr };
                this.0.update(&map.remove(&this.1).unwrap())
            }))
        } else {
            warn!("nbt reading event not found");
        }
    }
}

impl<T> Deref for Saved<T>
where
    T: Attach + crate::nbt::Update + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Saved<T>
where
    T: Attach + crate::nbt::Update + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn nbt_event_comp() -> Component<
    crate::Event<dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt_rc::error::Result<()>>,
> {
    Component(crate::Event::new(|listeners| {
        Box::new(move |map| {
            for listener in listeners {
                listener(map)?;
            }

            Ok(())
        })
    }))
}

#[inline]
pub fn nbt_save_event_comp_id() -> crate::Id {
    crate::Id::new("core", "nbt_save".to_string())
}

#[inline]
pub fn nbt_read_event_comp_id() -> crate::Id {
    crate::Id::new("core", "nbt_read".to_string())
}
