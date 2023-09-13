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
#[derive(Default)]
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
        Default::default()
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
        self.components.get(id).and_then(|value| {
            if value.1 == TypeId::of::<T>() {
                Some(unsafe { &*(&*value.0 as *const (dyn Attach + Send + Sync) as *const T) })
            } else {
                None
            }
        })
    }

    /// Get a mutable static typed component from this instance.
    pub fn get_mut<T>(&mut self, id: &crate::Id) -> Option<&mut T>
    where
        T: Attach + Send + Sync + 'static,
    {
        self.components.get_mut(id).and_then(|value| {
            if value.1 == TypeId::of::<T>() {
                Some(unsafe { &mut *(&mut *value.0 as *mut (dyn Attach + Send + Sync) as *mut T) })
            } else {
                None
            }
        })
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
            >>(&NET_SEND_ID)
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
        let Component(event) = self
            .get_mut::<Component<
                crate::MutOnly<
                    crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>,
                >,
            >>(&NET_RECV_ID)
            .expect("net recv event component not found");

        let mut hashmap = HashMap::<crate::Id, Bytes>::decode(buf)?;
        event.as_mut().invoker()(&mut hashmap)
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
                    dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt::error::Result<()>,
                >,
            >>(&NBT_SAVE_ID)
            .expect("net send event component not found");

        let mut hashmap = HashMap::new();

        use serde::ser::Error;
        event.invoker()(&mut hashmap).map_err(<S as serde::Serializer>::Error::custom)?;
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
            .get_mut::<Component<
                crate::MutOnly<
                    crate::Event<
                        dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt::error::Result<()>,
                    >,
                >,
            >>(&NBT_READ_ID)
            .expect("net recv event component not found");

        use serde::{de::Error, Deserialize};
        let mut hashmap = HashMap::deserialize(deserializer)?;
        event.as_mut().invoker()(&mut hashmap)
            .map_err(<D as serde::Deserializer<'_>>::Error::custom)
    }
}

impl From<ComponentsBuilder> for Components {
    fn from(value: ComponentsBuilder) -> Self {
        value.build()
    }
}

static ATTACH_EVENTS: parking_lot::RwLock<crate::Event<dyn Fn(TypeId, &mut Components)>> =
    parking_lot::RwLock::new(crate::Event::new(|listeners| {
        Box::new(move |type_id, components| {
            for listener in listeners {
                listener(type_id, components)
            }
        })
    }));

/// [`Components`] builder for creating with external features.
pub struct ComponentsBuilder {
    pub inner: Components,
}

impl ComponentsBuilder {
    /// Enable networking features for the instance.
    pub fn net_sync(mut self) -> Self {
        self.inner
            .register(net_send_event_comp_id(), net_event_comp());
        self.inner
            .register(net_recv_event_comp_id(), net_event_comp_mut());

        self
    }

    /// Enable nbt reading and writing feature for the instance.
    pub fn nbt_storing(mut self) -> Self {
        self.inner
            .register(nbt_save_event_comp_id(), nbt_event_comp());
        self.inner
            .register(nbt_read_event_comp_id(), nbt_event_comp_mut());

        self
    }

    pub fn register_defaults<T>(mut self) -> Self
    where
        T: 'static,
    {
        ATTACH_EVENTS.read().invoker()(TypeId::of::<T>(), &mut self.inner);
        self
    }

    /// Build this instance into [`Components`].
    #[inline]
    pub fn build(self) -> Components {
        self.inner
    }
}

/// Represents a simple component without extra
/// attach features, which has an empty
/// implementation of [`Attach`].
#[derive(Debug)]
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

impl<T> Encode for Component<T>
where
    T: Encode,
{
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.0.encode(buf)
    }
}

impl<T> NetSync for Component<T>
where
    T: NetSync,
{
    fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        self.0.read_buf(buf)
    }
}

impl<T> serde::Serialize for Component<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> crate::nbt::Update for Component<T>
where
    T: crate::nbt::Update,
{
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.update(deserializer)
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
#[derive(Debug)]
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
        >>(&*NET_SEND_ID)
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
            crate::MutOnly<
                crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>,
            >,
        >>(&*NET_RECV_ID)
        {
            event.as_mut().register(Box::new(move |map| {
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

impl<T> Encode for Synced<T>
where
    T: Attach + NetSync + 'static,
{
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.0.encode(buf)
    }
}

impl<T> NetSync for Synced<T>
where
    T: Attach + NetSync + 'static,
{
    fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        self.0.read_buf(buf)
    }
}

impl<T> serde::Serialize for Synced<T>
where
    T: Attach + NetSync + serde::Serialize + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> crate::nbt::Update for Synced<T>
where
    T: Attach + NetSync + crate::nbt::Update + 'static,
{
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.update(deserializer)
    }
}

fn net_event_comp(
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

fn net_event_comp_mut() -> Component<
    crate::MutOnly<crate::Event<dyn Fn(&mut HashMap<crate::Id, Bytes>) -> anyhow::Result<()>>>,
> {
    Component(crate::MutOnly::new(crate::Event::new(|listeners| {
        Box::new(move |map| {
            for listener in listeners {
                listener(map)?;
            }

            Ok(())
        })
    })))
}

#[inline]
fn net_send_event_comp_id() -> crate::Id {
    crate::Id::new("core", "net_send".to_string())
}

#[inline]
fn net_recv_event_comp_id() -> crate::Id {
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
#[derive(Debug)]
pub struct Stored<T>(pub T, pub crate::Id)
where
    T: Attach + crate::nbt::Update + 'static;

impl<T> Attach for Stored<T>
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
            crate::Event<dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt::error::Result<()>>,
        >>(&*NBT_SAVE_ID)
        {
            event.register(Box::new(move |map| {
                let this = unsafe { &*ptr };
                map.insert(
                    this.1.clone(),
                    this.0.serialize(&mut fastnbt::value::Serializer)?,
                );

                Ok(())
            }))
        } else {
            warn!("nbt saving event not found");
        }

        if let Some(Component(event)) = components.get_mut::<Component<
            crate::MutOnly<
                crate::Event<
                    dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt::error::Result<()>,
                >,
            >,
        >>(&*NBT_READ_ID)
        {
            event.as_mut().register(Box::new(move |map| {
                let this = unsafe { &mut *ptr };
                this.0.update(&map.remove(&this.1).unwrap())
            }))
        } else {
            warn!("nbt reading event not found");
        }
    }
}

impl<T> Deref for Stored<T>
where
    T: Attach + crate::nbt::Update + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Stored<T>
where
    T: Attach + crate::nbt::Update + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Encode for Stored<T>
where
    T: Attach + crate::nbt::Update + Encode + 'static,
{
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.0.encode(buf)
    }
}

impl<T> NetSync for Stored<T>
where
    T: Attach + crate::nbt::Update + NetSync + 'static,
{
    fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        self.0.read_buf(buf)
    }
}

impl<T> serde::Serialize for Stored<T>
where
    T: Attach + crate::nbt::Update + serde::Serialize + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> crate::nbt::Update for Stored<T>
where
    T: Attach + crate::nbt::Update + 'static,
{
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.update(deserializer)
    }
}

fn nbt_event_comp() -> Component<
    crate::Event<dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt::error::Result<()>>,
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

fn nbt_event_comp_mut() -> Component<
    crate::MutOnly<
        crate::Event<dyn Fn(&mut HashMap<crate::Id, NbtElement>) -> fastnbt::error::Result<()>>,
    >,
> {
    Component(crate::MutOnly::new(crate::Event::new(|listeners| {
        Box::new(move |map| {
            for listener in listeners {
                listener(map)?;
            }

            Ok(())
        })
    })))
}

#[inline]
fn nbt_save_event_comp_id() -> crate::Id {
    crate::Id::new("core", "nbt_save".to_string())
}

#[inline]
fn nbt_read_event_comp_id() -> crate::Id {
    crate::Id::new("core", "nbt_read".to_string())
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use crate::{
        component::Stored,
        nbt::Update,
        net::{Encode, NetSync},
    };

    use super::{Component, Components, Synced};

    #[test]
    fn register() {
        let mut components = Components::new();

        let id = crate::Id::new("test", "comp".to_string());
        components.register(id.clone(), Component(114_i32));

        assert_eq!(components.get::<Component<i32>>(&id).unwrap().0, 114);
        assert!(components.get::<Component<u8>>(&id).is_none());
    }

    #[test]
    fn net_sync() {
        let mut components_0 = Components::builder().net_sync().build();

        let id_0 = crate::Id::new("test", "comp0".to_string());
        components_0.register(id_0.clone(), Synced(Component(114_i32), id_0.clone()));
        let id_1 = crate::Id::new("test", "comp1".to_string());
        components_0.register(id_1.clone(), Synced(Component(514_i32), id_1.clone()));
        let id_2 = crate::Id::new("test", "comp2".to_string());
        components_0.register(id_2.clone(), Component(514_i32));

        let mut bytes = BytesMut::new();
        components_0.encode(&mut bytes).unwrap();
        let mut bytes: Bytes = bytes.into();

        let mut components_1 = Components::builder().net_sync().build();

        components_1.register(id_1.clone(), Synced(Component(0_i32), id_1.clone()));
        components_1.register(id_0.clone(), Synced(Component(0_i32), id_0.clone()));
        components_1.register(id_2.clone(), Component(0_i32));

        components_1.read_buf(&mut bytes).unwrap();

        assert_eq!(
            components_1
                .get::<Synced<Component<i32>>>(&id_0)
                .unwrap()
                .0
                 .0,
            114
        );

        assert_eq!(
            components_1
                .get::<Synced<Component<i32>>>(&id_1)
                .unwrap()
                .0
                 .0,
            514
        );

        assert_eq!(components_1.get::<Component<i32>>(&id_2).unwrap().0, 0);
    }

    #[test]
    fn nbt_rw() {
        let mut components_0 = Components::builder().nbt_storing().build();

        let id_0 = crate::Id::new("test", "comp0".to_string());
        components_0.register(id_0.clone(), Stored(Component(114_i32), id_0.clone()));
        let id_1 = crate::Id::new("test", "comp1".to_string());
        components_0.register(id_1.clone(), Stored(Component(514_i32), id_1.clone()));
        let id_2 = crate::Id::new("test", "comp2".to_string());
        components_0.register(id_2.clone(), Component(514_i32));

        let nbt = fastnbt::to_value(components_0).unwrap();

        let mut components_1 = Components::builder().nbt_storing().build();

        components_1.register(id_1.clone(), Stored(Component(0_i32), id_1.clone()));
        components_1.register(id_0.clone(), Stored(Component(0_i32), id_0.clone()));
        components_1.register(id_2.clone(), Component(0_i32));

        components_1.update(&nbt).unwrap();

        assert_eq!(
            components_1
                .get::<Stored<Component<i32>>>(&id_0)
                .unwrap()
                .0
                 .0,
            114
        );

        assert_eq!(
            components_1
                .get::<Stored<Component<i32>>>(&id_1)
                .unwrap()
                .0
                 .0,
            514
        );

        assert_eq!(components_1.get::<Component<i32>>(&id_2).unwrap().0, 0);
    }
}
