use erased_serde::Deserializer;
use once_cell::sync::Lazy;
use rimecraft_freezer::Freezer;
use serde::{Deserialize, Serialize};

use rimecraft_primitives::ErasedSerDeUpdate;

use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("hover event action not registered: {0}")]
    ActionNotRegistered(&'static str),
}

trait UpdDebug: ErasedSerDeUpdate + Debug {}
impl<T> UpdDebug for T where T: ?Sized + ErasedSerDeUpdate + Debug {}
erased_serde::serialize_trait_object!(UpdDebug);
rimecraft_primitives::update_trait_object!(UpdDebug);

/// Event performed when cursor is hovering on a `Text`.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.text.HoverEvent` (yarn).
#[derive(Clone)]
pub struct HoverEvent {
    contents: Arc<dyn UpdDebug + Send + Sync>,
    action: &'static ErasedAction,
}

impl HoverEvent {
    /// Creates a new event from given action and contents.
    pub fn new<T>(action: &Action<T>, contents: T) -> Self
    where
        T: ErasedSerDeUpdate + Debug + Hash + Send + Sync + 'static,
    {
        Self {
            contents: Arc::new(contents),
            action: ErasedAction::from_name(action.name)
                .ok_or(Error::ActionNotRegistered(action.name))
                .unwrap(),
        }
    }

    /// Gets the action of this event, with type erased.
    #[inline]
    pub fn action(&self) -> &ErasedAction {
        self.action
    }

    /// Gets the contents of this event from given type.
    ///
    /// If the type is correct, the return value will contain
    /// contents of this event.
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

impl Debug for HoverEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverEvent")
            .field("action", &self.action)
            .field("value", &self.contents)
            .finish()
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

impl<'de> Deserialize<'de> for HoverEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Struct {
            action: &'static ErasedAction,
            contents: serde_json::Value,
        }

        //TODO: if `contents` not found, deserialize text from field `value`.
        let Struct { action, contents } = Struct::deserialize(deserializer)?;

        // Deserializing contents.
        let mut contents_obj = (action.factory)();
        use serde::de::Error;
        contents_obj
            .erased_update(&mut <dyn Deserializer>::erase(contents))
            .map_err(D::Error::custom)?;

        Ok(Self {
            contents: contents_obj.into(),
            action,
        })
    }
}

/// Action of a [`HoverEvent`].
pub struct Action<T> {
    name: &'static str,
    parsable: bool,

    factory: fn() -> T,
}

/// Registers an action.
pub fn register_action<T: 'static>(action: Action<T>)
where
    T: ErasedSerDeUpdate + Debug + Send + Sync,
{
    ACTIONS.lock().insert(action.name, action.into());
}

impl<T> Hash for Action<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl<T> Copy for Action<T> {}
impl<T> Clone for Action<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// [`Action`] with type erased.
pub struct ErasedAction {
    name: &'static str,
    parsable: bool,

    factory: Box<dyn Fn() -> Box<dyn UpdDebug + Send + Sync> + Send + Sync>,
    /// [`TypeId`] of the target type.
    type_id: TypeId,
}

impl Hash for ErasedAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.type_id.hash(state);
    }
}

static ACTIONS: Lazy<Freezer<HashMap<&'static str, ErasedAction>>> =
    Lazy::new(|| Freezer::new(HashMap::new()));

impl<T> Action<T> {
    /// Creates a new action.
    #[inline]
    pub const fn new(name: &'static str, parsable: bool, constructor: fn() -> T) -> Self {
        Self {
            name,
            parsable,
            factory: constructor,
        }
    }

    /// Name of this action.
    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

    /// Whether this action is parsable.
    #[inline]
    pub fn is_parsable(&self) -> bool {
        self.parsable
    }
}

impl ErasedAction {
    /// Name of this action.
    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

    /// Whether this action is parsable.
    #[inline]
    pub fn is_parsable(&self) -> bool {
        self.parsable
    }

    /// Gets registered action from its name.
    #[inline]
    pub fn from_name(name: &str) -> Option<&'static Self> {
        ACTIONS.get_or_freeze().get(name)
    }
}

impl<T: 'static> From<Action<T>> for ErasedAction
where
    T: ErasedSerDeUpdate + Debug + Send + Sync,
{
    fn from(value: Action<T>) -> Self {
        Self {
            name: value.name,
            parsable: value.parsable,
            factory: Box::new(move || Box::new((value.factory)())),
            type_id: TypeId::of::<T>(),
        }
    }
}

impl<'a, T: 'static> TryFrom<&'a Action<T>> for &'static ErasedAction {
    type Error = Error;

    #[inline]
    fn try_from(value: &'a Action<T>) -> Result<Self, Self::Error> {
        ErasedAction::from_name(value.name).ok_or(Error::ActionNotRegistered(value.name))
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedHoverEventAction")
            .field("name", &self.name)
            .field("type", &self.type_id)
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
            Lazy::new(|| ACTIONS.get_or_freeze().keys().copied().collect());

        let value = String::deserialize(deserializer)?;

        use serde::de::Error;
        ErasedAction::from_name(&value).ok_or_else(|| D::Error::unknown_variant(&value, &VARIANTS))
    }
}
