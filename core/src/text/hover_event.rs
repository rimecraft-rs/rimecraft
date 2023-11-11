use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use rimecraft_event::Event;
use rimecraft_primitives::{ErasedSerDeUpdate, SerDeUpdate};

use std::{
    any::TypeId,
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

trait UpdDebug: ErasedSerDeUpdate + Debug {}
impl<T> UpdDebug for T where T: ?Sized + ErasedSerDeUpdate + Debug {}

erased_serde::serialize_trait_object!(UpdDebug);
rimecraft_primitives::update_trait_object!(UpdDebug);

pub struct HoverEvent {
    contents: Box<dyn UpdDebug + Send + Sync>,
    action: &'static ErasedAction,

    contents_hash: u64,
}

impl HoverEvent {
    pub fn new<T>(action: &'static Action<T>, contents: T) -> Self
    where
        T: ErasedSerDeUpdate + Debug + Hash + Send + Sync + 'static,
    {
        let mut hasher = DefaultHasher::new();
        contents.hash(&mut hasher);

        Self {
            contents: Box::new(contents),
            action: unsafe { std::mem::transmute(action) },
            contents_hash: hasher.finish(),
        }
    }

    #[inline]
    pub fn action(&self) -> &ErasedAction {
        self.action
    }

    pub fn value<T: 'static>(&self) -> Option<&T>
    where
        T: ErasedSerDeUpdate + Debug + Hash + Send + Sync,
    {
        if TypeId::of::<T>() == self.action.type_id {
            Some(unsafe { &*(&*self.contents as *const (dyn UpdDebug + Send + Sync) as *const T) })
        } else {
            None
        }
    }
}

impl Hash for HoverEvent {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.contents_hash);
        self.action.hash(state);
    }
}

impl Debug for HoverEvent {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HoverEvent{{action={:?},value={:?}}}",
            self.action, self.contents
        )
    }
}

impl PartialEq for HoverEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            self.action() as *const ErasedAction,
            other.action() as *const ErasedAction,
        )
    }
}

impl Serialize for HoverEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Struct<'a> {
            action: &'static ErasedAction,
            contents: &'a (dyn UpdDebug + Send + Sync),
        }

        Struct {
            action: self.action,
            contents: &*self.contents,
        }
        .serialize(serializer)
    }
}

impl SerDeUpdate for HoverEvent {
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Struct {
            action: &'static ErasedAction,
            contents: serde_json::Value,
        }

        let Struct { action, contents } = Struct::deserialize(deserializer)?;

        self.action = action;
        self.contents.update(contents).map_err(|err| {
            use serde::de::Error;
            D::Error::custom(err.to_string())
        })
    }
}

pub struct Action<T> {
    name: &'static str,
    parsable: bool,

    factory: fn() -> T,
}

impl<T> Hash for Action<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

pub struct ErasedAction {
    name: &'static str,
    parsable: bool,

    factory: Box<dyn Fn() -> Box<dyn UpdDebug + Send + Sync> + Send + Sync>,

    type_id: TypeId,
    type_name: &'static str,
}

impl Hash for ErasedAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.type_id.hash(state);
    }
}

/// An event to process actions on initialize.
static HE_ACTIONS: RwLock<Event<dyn Fn(&mut Vec<ErasedAction>)>> =
    RwLock::new(Event::new(|listeners| {
        Box::new(move |actions| {
            for listener in listeners {
                listener(actions)
            }
        })
    }));

static HE_MAPPING: Lazy<HashMap<String, ErasedAction>> = Lazy::new(|| {
    HE_ACTIONS.write().register(Box::new(|v| {
        v.append(&mut vec![
            //TODO: built-in actions
        ]);
    }));

    let mut vec = Vec::new();
    HE_ACTIONS.read().invoker()(&mut vec);
    vec.into_iter().map(|v| (v.name().to_owned(), v)).collect()
});

impl<T> Action<T> {
    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

    #[inline]
    pub fn is_parsable(&self) -> bool {
        self.parsable
    }
}

impl ErasedAction {
    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

    #[inline]
    pub fn is_parsable(&self) -> bool {
        self.parsable
    }

    #[inline]
    pub fn from_name(name: &str) -> Option<&'static Self> {
        HE_MAPPING.get(name)
    }
}

impl<T> From<Action<T>> for ErasedAction
where
    T: ErasedSerDeUpdate + Debug + Send + Sync,
{
    fn from(value: Action<T>) -> Self {
        Self {
            name: value.name,
            parsable: value.parsable,
            factory: Box::new(|| Box::new((value.factory)())),
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl<T> Debug for Action<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverEventAction")
            .field("name", &self.name)
            .finish()
    }
}

impl Debug for ErasedAction {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverEventErasedAction")
            .field("name", &self.name)
            .field("type", &self.type_name)
            .finish()
    }
}

impl<T> Serialize for Action<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.name().serialize(serializer)
    }
}

impl Serialize for ErasedAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.name().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for &'static ErasedAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static VARIANTS: Lazy<Vec<&'static str>> =
            Lazy::new(|| HE_MAPPING.iter().map(|v| v.0.as_str()).collect());

        let value = String::deserialize(deserializer)?;

        use serde::de::Error;
        ErasedAction::from_name(&String::deserialize(deserializer)?)
            .ok_or_else(|| D::Error::unknown_variant(&value, &VARIANTS))
    }
}
