use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use tracing::{trace_span, warn};

use crate::net::{Decode, Encode, NetSync};

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

pub struct Components {
    components: HashMap<crate::Id, (Box<dyn Attach + Send + Sync>, TypeId)>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    #[inline]
    pub fn builder() -> ComponentsBuilder {
        ComponentsBuilder { inner: Self::new() }
    }

    pub fn attach<T>(&mut self, id: crate::Id, component: T)
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

pub struct ComponentsBuilder {
    inner: Components,
}

impl ComponentsBuilder {
    pub fn net_sync(mut self) -> Self {
        self.inner
            .attach(net_send_event_comp_id(), net_send_event_comp());
        self.inner
            .attach(net_recv_event_comp_id(), net_recv_event_comp());
        self
    }

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
                let mut bytes = map.insert(this.1.clone(), Bytes::new()).unwrap();

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

/// Creates a new event that handle network packet buf sending mapping,
/// which component id is `core:net_send`.
pub fn net_send_event_comp(
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

/// Creates a new event that handle network packet buf receiving mapping,
/// which component id is `core:net_recv`.
pub fn net_recv_event_comp(
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
pub fn net_recv_event_comp_id() -> crate::Id {
    crate::Id::new("core", "net_recv".to_string())
}
