use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use rimecraft_edcode::Encode;
use rimecraft_event::Event;
use rimecraft_primitives::{Id, SerDeUpdate};
use tracing::{trace_span, warn};

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
    components: HashMap<Id, (Box<dyn Attach + Send + Sync>, TypeId, &'static str)>,
}

impl Components {
    /// Creates a new empty components instance,
    /// without networking and saving features.
    ///
    /// To create with external features,
    /// see [`Self::builder()`].
    #[inline]
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
    pub fn register<T>(&mut self, id: Id, component: T)
    where
        T: Attach + Send + Sync + 'static,
    {
        debug_assert!(
            !self.components.contains_key(&id),
            "component with id {id} already exist in this components!"
        );

        let mut boxed = Box::new(component);
        boxed.on_attach(self);
        self.components
            .insert(id, (boxed, TypeId::of::<T>(), type_name::<T>()));
    }

    /// Get a static typed component from this instance.
    #[inline]
    pub fn get<T>(&self, id: &Id) -> Result<&T, ComponentsError>
    where
        T: Attach + Send + Sync + 'static,
    {
        let value = self
            .components
            .get(id)
            .ok_or_else(|| ComponentsError::IdNotFound(id.to_owned()))?;
        if value.1 == TypeId::of::<T>() {
            Ok(unsafe { &*(&*value.0 as *const (dyn Attach + Send + Sync) as *const T) })
        } else {
            Err(ComponentsError::TypeNotMatch {
                expected: type_name::<T>(),
                found: value.2,
                id: id.to_owned(),
            })
        }
    }

    /// Get a mutable static typed component from this instance.
    #[inline]
    pub fn get_mut<T>(&mut self, id: &Id) -> Result<&mut T, ComponentsError>
    where
        T: Attach + Send + Sync + 'static,
    {
        let value = self
            .components
            .get_mut(id)
            .ok_or_else(|| ComponentsError::IdNotFound(id.to_owned()))?;
        if value.1 == TypeId::of::<T>() {
            Ok(unsafe { &mut *(&mut *value.0 as *mut (dyn Attach + Send + Sync) as *mut T) })
        } else {
            Err(ComponentsError::TypeNotMatch {
                expected: type_name::<T>(),
                found: value.2,
                id: id.to_owned(),
            })
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ComponentsError {
    #[error("component with id {0} not found")]
    IdNotFound(Id),
    #[error("component with id {id} found, but its type is {found}, expected {expected}")]
    TypeNotMatch {
        expected: &'static str,
        found: &'static str,

        id: Id,
    },
}

impl Encode for Components {
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        let Component(event) = self.get::<Component<BytesPEvent>>(&NET_SEND_ID).unwrap();

        let mut hashmap = HashMap::new();
        event.invoker()(&mut hashmap)?;
        hashmap.encode(buf)
    }
}

impl rimecraft_edcode::Update for Components {
    fn update<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        use rimecraft_edcode::Decode;

        let Component(event) = self
            .get_mut::<Component<BytesPEvent>>(&NET_RECV_ID)
            .unwrap();

        let mut hashmap = HashMap::<Id, Bytes>::decode(buf)?;
        event.invoker()(&mut hashmap)
    }
}

impl serde::Serialize for Components {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Component(event) = self.get::<Component<ValuePEvent>>(&NBT_SAVE_ID).unwrap();

        let mut hashmap = HashMap::new();

        use serde::ser::Error;
        event.invoker()(&mut hashmap).map_err(<S as serde::Serializer>::Error::custom)?;
        hashmap.serialize(serializer)
    }
}

impl SerDeUpdate for Components {
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Component(event) = self
            .get_mut::<Component<ValuePEvent>>(&NBT_READ_ID)
            .unwrap();

        use serde::{de::Error, Deserialize};
        let mut hashmap = HashMap::deserialize(deserializer)?;
        event.invoker()(&mut hashmap).map_err(<D as serde::Deserializer<'_>>::Error::custom)
    }
}

impl From<ComponentsBuilder> for Components {
    #[inline]
    fn from(value: ComponentsBuilder) -> Self {
        value.build()
    }
}

type CompPEvent = Event<dyn Fn(TypeId, &mut Components)>;

static ATTACH_EVENTS: parking_lot::RwLock<CompPEvent> =
    parking_lot::RwLock::new(Event::new(|listeners| {
        Box::new(move |type_id, components| {
            for listener in listeners {
                listener(type_id, components)
            }
        })
    }));

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

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Component<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Encode for Component<T>
where
    T: Encode,
{
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.0.encode(buf)
    }
}

impl<T> rimecraft_edcode::Update for Component<T>
where
    T: rimecraft_edcode::Update,
{
    #[inline]
    fn update<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        self.0.update(buf)
    }
}

impl<T> serde::Serialize for Component<T>
where
    T: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> SerDeUpdate for Component<T>
where
    T: SerDeUpdate,
{
    #[inline]
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

static NET_SEND_ID: once_cell::sync::Lazy<Id> = once_cell::sync::Lazy::new(net_send_event_comp_id);
static NET_RECV_ID: once_cell::sync::Lazy<Id> = once_cell::sync::Lazy::new(net_recv_event_comp_id);

/// Represents a component that able to sync by
/// networking methods, through [`SerDeUpdate`] trait.
///
/// The `1` field is the component id which is used
/// to be registered into components.
#[derive(Debug)]
pub struct Synced<T>(pub T, pub Id)
where
    T: Attach + rimecraft_edcode::Update + 'static;

impl<T> Attach for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + 'static,
{
    fn on_attach(&mut self, components: &mut Components) {
        self.0.on_attach(components);

        let ptr = self as *mut Self;

        let span = trace_span!(
            "attach synced component",
            comp_type = std::any::type_name::<T>()
        );

        let _ = span.enter();

        match components
            .get_mut::<Component<Event<dyn Fn(&mut HashMap<Id, Bytes>) -> anyhow::Result<()>>>>(
                &NET_SEND_ID,
            ) {
            Ok(Component(event)) => event.register(Box::new(move |map| {
                let this = unsafe { &*ptr };

                map.insert(this.1.clone(), {
                    let mut bytes_mut = bytes::BytesMut::new();
                    this.0.encode(&mut bytes_mut)?;

                    bytes_mut.into()
                });

                Ok(())
            })),
            Err(err) => {
                warn!("network sending event not found: {err}");
            }
        }

        match components.get_mut::<Component<BytesPEvent>>(&NET_RECV_ID) {
            Ok(Component(event)) => event.register(Box::new(move |map| {
                let this = unsafe { &mut *ptr };
                let mut bytes = map.remove(&this.1).unwrap();

                this.0.update(&mut bytes)
            })),
            Err(err) => {
                warn!("network receiving event not found: {err}");
            }
        }
    }
}

impl<T> Deref for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + 'static,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + 'static,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Encode for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + 'static,
{
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.0.encode(buf)
    }
}

impl<T> rimecraft_edcode::Update for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + 'static,
{
    #[inline]
    fn update<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        self.0.update(buf)
    }
}

impl<T> serde::Serialize for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + serde::Serialize + 'static,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> SerDeUpdate for Synced<T>
where
    T: Attach + rimecraft_edcode::Update + SerDeUpdate + 'static,
{
    #[inline]
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerDeUpdate::update(&mut self.0, deserializer)
    }
}

type BytesPEvent = Event<dyn Fn(&mut HashMap<Id, Bytes>) -> anyhow::Result<()>>;

#[inline]
fn net_event_comp() -> Component<BytesPEvent> {
    Component(Event::new(|listeners| {
        Box::new(move |map| {
            for listener in listeners {
                listener(map)?;
            }

            Ok(())
        })
    }))
}

#[inline]
fn net_send_event_comp_id() -> Id {
    Id::new("core", "net_send".to_string())
}

#[inline]
fn net_recv_event_comp_id() -> Id {
    Id::new("core", "net_recv".to_string())
}

static NBT_SAVE_ID: once_cell::sync::Lazy<Id> = once_cell::sync::Lazy::new(nbt_save_event_comp_id);
static NBT_READ_ID: once_cell::sync::Lazy<Id> = once_cell::sync::Lazy::new(nbt_read_event_comp_id);

/// Represents a component that able to be stored
/// by nbt, through [`SerDeUpdate`] trait.
///
/// The `1` field is the component id which is used
/// to be registered into components.
#[derive(Debug)]
pub struct Stored<T>(pub T, pub Id)
where
    T: Attach + SerDeUpdate + 'static;

impl<T> Attach for Stored<T>
where
    T: Attach + SerDeUpdate + 'static,
{
    fn on_attach(&mut self, components: &mut Components) {
        self.0.on_attach(components);

        let ptr = self as *mut Self;

        let span = trace_span!(
            "attach saved component",
            comp_type = std::any::type_name::<T>()
        );

        let _ = span.enter();

        match components.get_mut::<Component<
            Event<dyn Fn(&mut HashMap<Id, fastnbt::Value>) -> fastnbt::error::Result<()>>,
        >>(&NBT_SAVE_ID)
        {
            Ok(Component(event)) => event.register(Box::new(move |map| {
                let this = unsafe { &*ptr };
                map.insert(
                    this.1.clone(),
                    this.0.serialize(&mut fastnbt::value::Serializer)?,
                );

                Ok(())
            })),
            Err(err) => {
                warn!("nbt saving event not found: {err}");
            }
        }

        match components.get_mut::<Component<ValuePEvent>>(&NBT_READ_ID) {
            Ok(Component(event)) => event.register(Box::new(move |map| {
                let this = unsafe { &mut *ptr };
                this.0.update(&map.remove(&this.1).unwrap())
            })),
            Err(err) => {
                warn!("nbt reading event not found: {err}");
            }
        }
    }
}

impl<T> Deref for Stored<T>
where
    T: Attach + SerDeUpdate + 'static,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Stored<T>
where
    T: Attach + SerDeUpdate + 'static,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Encode for Stored<T>
where
    T: Attach + SerDeUpdate + Encode + 'static,
{
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.0.encode(buf)
    }
}

impl<T> rimecraft_edcode::Update for Stored<T>
where
    T: Attach + SerDeUpdate + rimecraft_edcode::Update + 'static,
{
    #[inline]
    fn update<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        rimecraft_edcode::Update::update(&mut self.0, buf)
    }
}

impl<T> serde::Serialize for Stored<T>
where
    T: Attach + SerDeUpdate + serde::Serialize + 'static,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> SerDeUpdate for Stored<T>
where
    T: Attach + SerDeUpdate + 'static,
{
    #[inline]
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

type ValuePEvent = Event<dyn Fn(&mut HashMap<Id, fastnbt::Value>) -> fastnbt::error::Result<()>>;

fn nbt_event_comp() -> Component<ValuePEvent> {
    Component(Event::new(|listeners| {
        Box::new(move |map| {
            for listener in listeners {
                listener(map)?;
            }

            Ok(())
        })
    }))
}

#[inline]
fn nbt_save_event_comp_id() -> Id {
    Id::new("core", "nbt_save".to_string())
}

#[inline]
fn nbt_read_event_comp_id() -> Id {
    Id::new("core", "nbt_read".to_string())
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use rimecraft_primitives::Id;

    use crate::component::Stored;

    use super::{Component, Components, Synced};

    #[test]
    fn register() {
        let mut components = Components::new();

        let id = Id::new("test", "comp".to_string());
        components.register(id.clone(), Component(114_i32));

        assert_eq!(components.get::<Component<i32>>(&id).unwrap().0, 114);
        assert!(components.get::<Component<u8>>(&id).is_err());
    }

    #[test]
    fn net_sync() {
        let mut components_0 = Components::builder().net_sync().build();

        let id_0 = Id::new("test", "comp0".to_string());
        components_0.register(id_0.clone(), Synced(Component(114_i32), id_0.clone()));
        let id_1 = Id::new("test", "comp1".to_string());
        components_0.register(id_1.clone(), Synced(Component(514_i32), id_1.clone()));
        let id_2 = Id::new("test", "comp2".to_string());
        components_0.register(id_2.clone(), Component(514_i32));

        let mut bytes = BytesMut::new();
        rimecraft_edcode::Encode::encode(&components_0, &mut bytes).unwrap();
        let mut bytes: Bytes = bytes.into();

        let mut components_1 = Components::builder().net_sync().build();

        components_1.register(id_1.clone(), Synced(Component(0_i32), id_1.clone()));
        components_1.register(id_0.clone(), Synced(Component(0_i32), id_0.clone()));
        components_1.register(id_2.clone(), Component(0_i32));

        rimecraft_edcode::Update::update(&mut components_1, &mut bytes).unwrap();

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

        let id_0 = Id::new("test", "comp0".to_string());
        components_0.register(id_0.clone(), Stored(Component(114_i32), id_0.clone()));
        let id_1 = Id::new("test", "comp1".to_string());
        components_0.register(id_1.clone(), Stored(Component(514_i32), id_1.clone()));
        let id_2 = Id::new("test", "comp2".to_string());
        components_0.register(id_2.clone(), Component(514_i32));

        let nbt = fastnbt::to_value(components_0).unwrap();

        let mut components_1 = Components::builder().nbt_storing().build();

        components_1.register(id_1.clone(), Stored(Component(0_i32), id_1.clone()));
        components_1.register(id_0.clone(), Stored(Component(0_i32), id_0.clone()));
        components_1.register(id_2.clone(), Component(0_i32));

        rimecraft_primitives::SerDeUpdate::update(&mut components_1, &nbt).unwrap();

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
