//! Serde support for [`Attachments`].

use std::{
    collections::HashMap,
    convert::Infallible,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::ser::SerializeMap;

use crate::{AsAttachment, AsAttachmentMut, Attach, Attachments};

type SerState<K> = Vec<(K, Box<dyn Fn() -> Box<dyn AsErasedSerialize>>)>;
type UpdateState<K> = HashMap<K, Box<dyn Fn() -> Box<dyn AsErasedUpdate>>>;

pub(crate) struct State<K> {
    ser: SerState<K>,
    update: UpdateState<K>,
}

impl<K> State<K> {
    #[inline]
    pub fn new() -> Self {
        Self {
            ser: Vec::new(),
            update: HashMap::new(),
        }
    }
}

impl<K> Default for State<K> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent attachment that can be serialized
/// and deserialized.
pub struct Persistent<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> Persistent<T> {
    /// Creates a new [`Persistent`] from given value.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Returns the raw inner value.
    #[inline]
    pub fn raw(&self) -> Arc<RwLock<T>> {
        self.inner.clone()
    }
}

impl<T, K> Attach<K> for Persistent<T>
where
    T: serde::Serialize + for<'de> rimecraft_serde_update::Update<'de> + Send + Sync + 'static,
    K: Clone + Hash + Eq + Send + Sync + 'static,
{
    type Attached = Self;

    type Error = Infallible;

    fn attach(
        &mut self,
        attachments: &mut crate::Attachments<K>,
        key: &K,
    ) -> Result<(), Self::Error> {
        let inner = self.inner.clone();
        attachments
            .serde_state
            .ser
            .push((key.clone(), Box::new(move || Box::new(inner.read_arc()))));
        let inner = self.inner.clone();
        attachments
            .serde_state
            .update
            .insert(key.clone(), Box::new(move || Box::new(inner.write_arc())));
        Ok(())
    }

    #[inline]
    fn into_attached(self) -> Self::Attached {
        self
    }
}

impl<'a, T: 'a> AsAttachment<'a> for Persistent<T> {
    type Target = T;

    type Output = PersistentGuard<'a, T>;

    #[inline]
    fn as_attachment(&'a self) -> Self::Output {
        PersistentGuard {
            inner: self.inner.read(),
        }
    }
}

impl<'a, T: 'a> AsAttachmentMut<'a> for Persistent<T> {
    type Output = PersistentGuardMut<'a, T>;

    #[inline]
    fn as_attachment_mut(&'a mut self) -> <Self as AsAttachmentMut<'a>>::Output {
        PersistentGuardMut {
            inner: self.inner.write(),
        }
    }
}

trait AsErasedSerialize {
    fn as_serialize(&self) -> &dyn erased_serde::Serialize;
}

impl<T> AsErasedSerialize for T
where
    T: Deref,
    <T as Deref>::Target: serde::Serialize + Sized + 'static,
{
    #[inline]
    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        &**self
    }
}

trait AsErasedUpdate {
    fn as_update(&mut self) -> &mut dyn for<'de> rimecraft_serde_update::erased::ErasedUpdate<'de>;
}

impl<T> AsErasedUpdate for T
where
    T: DerefMut,
    <T as Deref>::Target: for<'de> rimecraft_serde_update::Update<'de> + Sized + 'static,
{
    #[inline]
    fn as_update(&mut self) -> &mut dyn for<'de> rimecraft_serde_update::erased::ErasedUpdate<'de> {
        &mut **self
    }
}

/// A guard that can be used to access the inner value.
#[derive(Debug)]
pub struct PersistentGuard<'a, T> {
    inner: RwLockReadGuard<'a, T>,
}

impl<'a, T> Deref for PersistentGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A mutable guard that can be used to access the inner value.
#[derive(Debug)]
pub struct PersistentGuardMut<'a, T> {
    inner: RwLockWriteGuard<'a, T>,
}

impl<'a, T> Deref for PersistentGuardMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for PersistentGuardMut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K: serde::Serialize> serde::Serialize for Attachments<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.serde_state.ser.len()))?;
        for (key, value) in &self.serde_state.ser {
            let ser = value();
            map.serialize_entry(key, ser.as_serialize())?;
        }
        map.end()
    }
}

impl<'de, K> rimecraft_serde_update::Update<'de> for Attachments<K>
where
    K: serde::Deserialize<'de> + rimecraft_serde_update::Update<'de> + Hash + Eq,
{
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a, K> {
            state: &'a UpdateState<K>,
        }

        impl<'a, 'de, K> serde::de::Visitor<'de> for Visitor<'a, K>
        where
            K: serde::Deserialize<'de> + Hash + Eq,
        {
            type Value = ();

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<K>()? {
                    let Some(update) = self.state.get(&key) else {
                        continue;
                    };

                    struct Seed<'a, 'de>(
                        &'a mut dyn rimecraft_serde_update::erased::ErasedUpdate<'de>,
                    );

                    impl<'de, 'a> serde::de::DeserializeSeed<'de> for Seed<'a, 'de> {
                        type Value = ();

                        #[inline]
                        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            self.0
                                .erased_update(&mut <dyn erased_serde::Deserializer>::erase(
                                    deserializer,
                                ))
                                .map_err(serde::de::Error::custom)
                        }
                    }

                    let mut update = update();
                    map.next_value_seed(Seed(update.as_update()))?;
                }
                Ok(())
            }
        }

        deserializer.deserialize_map(Visitor {
            state: &self.serde_state.update,
        })
    }
}
