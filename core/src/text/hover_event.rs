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
    hash::{Hash, Hasher}, marker::PhantomData,
};

trait UpdDebug: ErasedSerDeUpdate + Debug {}
impl<T> UpdDebug for T where T: ?Sized + ErasedSerDeUpdate + Debug {}

erased_serde::serialize_trait_object!(UpdDebug);
rimecraft_primitives::update_trait_object!(UpdDebug);

pub struct HoverEvent {
    contents: Box<dyn UpdDebug + Send + Sync>,
    action: &'static HoverEventAction<()>,
    hash: u64,
}

impl HoverEvent {
    pub fn new<T>(action: &'static HoverEventAction<T>, contents: T) -> Self
    where
        T: ErasedSerDeUpdate + Debug + Hash + Send + Sync + 'static,
    {
        let mut hasher = DefaultHasher::new();
        contents.hash(&mut hasher);

        Self {
            contents: Box::new(contents),
            action: unsafe { std::mem::transmute(action) },
            hash: hasher.finish(),
        }
    }

    #[inline]
    pub fn action(&self) -> &'static HoverEventAction<()> {
        &self.action
    }

    pub fn value<T: 'static>(&self) -> Option<&T> {
        if TypeId::of::<T>() == self.action.type_id {
            Some(unsafe {
                &*(&*self.contents as *const (dyn UpdDebug + Send + Sync) as *const T)
            })
        } else {
            None
        }
    }
}

impl Hash for HoverEvent {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
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
    fn eq(&self, other: &Self) -> bool {
        self.action() == other.action()
    }
}

impl Serialize for HoverEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Struct<'a> {
            action: &'static HoverEventAction<()>,
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
            action: &'static HoverEventAction<()>,
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

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct HoverEventAction<T> {
    name: Cow<'static, str>,
    parsable: bool,

    type_id: TypeId,
    factory: fn() -> Box<dyn UpdDebug + Send + Sync>,

    _marker: PhantomData<fn() -> T>,
}

/// An event to process actions on initialize.
static HE_ACTIONS: RwLock<Event<dyn Fn(&mut Vec<HoverEventAction<()>>)>> =
    RwLock::new(Event::new(|listeners| {
        Box::new(move |actions| {
            for listener in listeners {
                listener(actions)
            }
        })
    }));

static HE_MAPPING: Lazy<HashMap<String, HoverEventAction<()>>> = Lazy::new(|| {
    HE_ACTIONS.write().register(Box::new(|v| {
        v.append(&mut vec![
            HoverEventAction::SHOW_TEXT,
            HoverEventAction::SHOW_ITEM,
            HoverEventAction::SHOW_ENTITY,
        ]);
    }));

    let mut vec = Vec::new();
    HE_ACTIONS.read().invoker()(&mut vec);
    vec.into_iter().map(|v| (v.name().to_owned(), v)).collect()
});

impl<T> HoverEventAction<T> {
    pub const SHOW_TEXT: Self = Self {
        name: Cow::Borrowed("show_text"),
        parsable: true,
    };

    pub const SHOW_ITEM: Self = Self {
        name: Cow::Borrowed("show_item"),
        parsable: true,
    };

    pub const SHOW_ENTITY: Self = Self {
        name: Cow::Borrowed("show_entity"),
        parsable: true,
    };

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
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

impl<T> Debug for HoverEventAction<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverEventAction")
            .field("name", &self.name)
            .finish()
    }
}

impl<T> Serialize for HoverEventAction<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.name().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for &'static HoverEventAction<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static VARIANTS: Lazy<Vec<&'static str>> =
            Lazy::new(|| HE_MAPPING.iter().map(|v| v.0.as_str()).collect());

        let value = String::deserialize(deserializer)?;

        use serde::de::Error;

        HoverEventAction::from_name(&value)
            .ok_or_else(|| D::Error::unknown_variant(&value, &VARIANTS))
    }
}
