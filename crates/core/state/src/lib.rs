//! Minecraft state holders.
//!
//! This corresponds to `net.minecraft.state` in `yarn`.

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    ptr::NonNull,
    sync::OnceLock,
};

use ahash::AHashMap;
use property::{BiIndex, ErasedProperty, Property, UnObjSafeErasedWrap as _, Wrap};

#[cfg(feature = "regex")]
use regex::Regex;
#[cfg(not(feature = "regex"))]
use regex_lite::Regex;

use crate::property::ErasedWrap;

pub mod property;

// <property> -> <<value> -> <state>>
type Table<'a, T> = AHashMap<ErasedProperty<'a>, AHashMap<isize, NonNull<T>>>;

/// State of an object.
pub struct State<'a, T> {
    pub(crate) entries: AHashMap<ErasedProperty<'a>, isize>,
    table: OnceLock<Table<'a, Self>>,
    data: T,
}

impl<T> State<'_, T> {
    /// Gets the current value of given property in this state.
    #[inline]
    pub fn get<V, W>(&self, prop: &Property<'_, W>) -> Option<V>
    where
        W: BiIndex<V>,
    {
        self.entries
            .get(prop.name())
            .and_then(|&index| prop.wrap.index(index))
    }

    /// Gets the state of this state with given property `prop` cycled.
    ///
    /// # Errors
    ///
    /// Errors if the property `prop` is not present in this state.
    ///
    /// # Panics
    ///
    /// - Panics if the target state was dropped.
    /// - Panics if this state is not fully initialized.
    pub fn cycle<V, W>(&self, prop: &Property<'_, W>) -> Result<&Self, Error>
    where
        W: BiIndex<V>,
        for<'w> &'w W: IntoIterator<Item = V>,
    {
        let index = *self
            .entries
            .get(prop.name())
            .ok_or_else(|| Error::PropertyNotFound(prop.name().to_owned()))?;
        let Some(next) = obtain_next(
            index,
            (&prop.wrap)
                .into_iter()
                .filter_map(|value| prop.wrap.index_of(&value)),
        ) else {
            return Ok(self);
        };
        if next == index {
            Ok(self)
        } else {
            self.table
                .get()
                .expect("state not initialized")
                .get(prop.name())
                .ok_or_else(|| Error::PropertyNotFound(prop.name().to_owned()))
                .and_then(|map| map.get(&next).ok_or(Error::ValueNotFound(index)))
                .map(|ptr| unsafe { ptr.as_ref() })
        }
    }

    /// Gets the state of this state with given property `prop` set to `value`.
    ///
    /// # Errors
    ///
    /// - Errors if the property `prop` is not present in this state.
    /// - Errors if the value `value` is not present in the property `prop`.
    ///
    /// # Panics
    ///
    /// - Panics if the target state was dropped.
    /// - Panics if this state is not fully initialized.
    pub fn with<V, W>(&self, prop: &Property<'_, W>, value: V) -> Result<&Self, Error>
    where
        W: BiIndex<V>,
    {
        let index = *self
            .entries
            .get(prop.name())
            .ok_or_else(|| Error::PropertyNotFound(prop.name().to_owned()))?;
        let value = prop.wrap.index_of(&value).ok_or(Error::InvalidValue)?;
        if value == index {
            Ok(self)
        } else {
            self.table
                .get()
                .expect("state not initialized")
                .get(prop.name())
                .ok_or_else(|| Error::PropertyNotFound(prop.name().to_owned()))
                .and_then(|map| map.get(&value).ok_or(Error::ValueNotFound(index)))
                .map(|ptr| unsafe { ptr.as_ref() })
        }
    }

    /// Whether this state contains given property.
    #[inline]
    pub fn contains<W, V>(&self, prop: &Property<'_, W>) -> bool {
        self.entries.contains_key(prop.name())
    }

    /// Gets the data of this state.
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }
}

fn obtain_next(value: isize, mut iter: impl Iterator<Item = isize>) -> Option<isize> {
    let mut first = None;
    while let Some(next) = iter.next() {
        if first.is_none() {
            first = Some(next);
        }
        if next == value {
            return iter.next().or(first);
        }
    }
    iter.next()
}

impl<T: Debug> Debug for State<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("entries", &self.entries)
            .field("data", &self.data)
            .finish()
    }
}

/// Immutable instance of states.
///
/// See [`StatesMut`] for creating a new instance.
#[derive(Debug)]
#[doc(alias = "StateManager")]
pub struct States<'a, T> {
    states: Vec<NonNull<State<'a, T>>>,
    #[allow(unused)]
    props: BTreeMap<&'a str, ErasedProperty<'a>>,
}

impl<'a, T> States<'a, T>
where
    T: Clone,
{
    fn new<I>(props: I, data: T) -> Self
    where
        I: IntoIterator<Item = ErasedProperty<'a>>,
    {
        let props: BTreeMap<_, _> = props.into_iter().map(|prop| (prop.name, prop)).collect();
        let mut iter: Vec<Vec<(ErasedProperty<'a>, isize)>> = vec![Vec::new()];
        for prop in props.values() {
            iter = iter
                .into_iter()
                .flat_map(|lx| {
                    prop.wrap
                        .erased_iter()
                        .map(|val| {
                            let mut lx = lx.clone();
                            lx.push((prop.clone(), val));
                            lx
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
        }
        let list = iter
            .into_iter()
            .map(|vec| vec.into_iter().collect::<AHashMap<_, _>>())
            .map(|entries| {
                NonNull::new(Box::into_raw(Box::new(State {
                    entries,
                    table: OnceLock::new(),
                    data: data.clone(),
                })))
                .expect("failed to allocate state")
            })
            .collect::<Vec<_>>();

        // Initialize tables
        for state in list.iter() {
            let state = unsafe { state.as_ref() };
            let mut table: Table<'a, State<'a, T>> = Table::new();
            for (prop, s_val) in state.entries.iter() {
                let mut row = AHashMap::new();
                for val in prop.wrap.erased_iter().filter(|v| v != s_val) {
                    let Some(s) = list.iter().find(|s| {
                        let s = unsafe { s.as_ref() };
                        s.entries.iter().all(|(p, v)| {
                            if p == prop {
                                *v == val
                            } else {
                                v == state.entries.get(p).unwrap()
                            }
                        })
                    }) else {
                        continue;
                    };
                    row.insert(val, *s);
                }
                table.insert(prop.clone(), row);
            }
            state.table.set(table).expect("state already initialized");
        }

        Self {
            states: list,
            props,
        }
    }
}

impl<'a, T> States<'a, T> {
    /// Gets all states of this state.
    #[inline]
    pub fn states(&self) -> &[NonNull<State<'a, T>>] {
        &self.states
    }

    /// Gets the default state.
    ///
    /// # Panics
    ///
    /// Panics if no state is available.
    #[inline]
    pub fn default_state(&self) -> &State<'a, T> {
        unsafe { self.states.first().expect("no state available").as_ref() }
    }

    /// Gets the length of states.
    #[inline]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether the states is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl<T> Drop for States<'_, T> {
    fn drop(&mut self) {
        for state in self.states.iter() {
            drop(unsafe { Box::from_raw(state.as_ptr()) });
        }
    }
}

/// Mutable instance of [`States`].
#[derive(Debug)]
pub struct StatesMut<'a, T> {
    props: Vec<ErasedProperty<'a>>,
    data: T,
}

impl<'a, T> StatesMut<'a, T> {
    /// Creates a new states with given data.
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            props: Vec::new(),
            data,
        }
    }

    /// Adds a property to the states.
    ///
    /// # Errors
    ///
    /// - Errors if the property name is invalid.
    /// - Errors if the property contains <= 1 possible values.
    /// - Errors if the states contains duplicated properties.
    /// - Errors if any of the value name is invalid.
    #[allow(clippy::missing_panics_doc)]
    pub fn add<'p, W, G>(&mut self, prop: &'a Property<'p, W>) -> Result<(), Error>
    where
        W: Wrap<G> + BiIndex<G> + Eq + Send + Sync + 'p,
        for<'w> &'w W: IntoIterator<Item = G>,
    {
        static NAME_PAT: OnceLock<Regex> = OnceLock::new();
        let reg = NAME_PAT.get_or_init(|| Regex::new(r"^[a-z0-9_]+$").unwrap());
        if !reg.is_match(prop.name()) {
            return Err(Error::InvalidPropertyName(prop.name().to_owned()));
        }
        let mut len = 0;
        for val in prop.wrap.erased_iter_typed() {
            len += 1;
            let name = prop.wrap.erased_to_name(val).expect("invalid value");
            if !reg.is_match(&name) {
                return Err(Error::InvalidValueName {
                    property: prop.name().to_owned(),
                    value: name.into_owned(),
                });
            }
        }
        if len <= 1 {
            return Err(Error::PropertyContainsOneOrNoValue(prop.name().to_owned()));
        }
        if self.props.iter().any(|p| p.name == prop.name()) {
            return Err(Error::DuplicatedProperty(prop.name().to_owned()));
        }

        self.props.push(prop.into());
        Ok(())
    }
}

impl<'a, T> StatesMut<'a, T>
where
    T: Clone,
{
    /// Freezes the state.
    #[inline]
    pub fn freeze(self) -> States<'a, T> {
        States::new(self.props, self.data)
    }
}

impl<'a, T> From<StatesMut<'a, T>> for States<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(value: StatesMut<'a, T>) -> Self {
        value.freeze()
    }
}

/// Error type for state operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The property was not found in the state.
    PropertyNotFound(String),
    /// The table was not found in the state.
    TableNotPresent,
    /// The value was not found in the property.
    ValueNotFound(isize),
    /// The value is invalid.
    InvalidValue,

    /// The property name is invalid.
    InvalidPropertyName(String),
    /// The property contains <= 1 possible values.
    PropertyContainsOneOrNoValue(String),
    /// The value name is invalid.
    InvalidValueName {
        /// The property name.
        property: String,
        /// The value name.
        value: String,
    },
    /// The states contains duplicated properties.
    DuplicatedProperty(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PropertyNotFound(prop) => write!(f, "property not found: {prop}"),
            Error::TableNotPresent => write!(f, "table not present"),
            Error::ValueNotFound(value) => write!(f, "value not found: {value}"),
            Error::InvalidValue => write!(f, "invalid value"),
            Error::InvalidPropertyName(name) => write!(f, "invalid property name: {name}"),
            Error::PropertyContainsOneOrNoValue(prop) => {
                write!(f, "property {prop} contains <= 1 possible values")
            }
            Error::InvalidValueName { property, value } => {
                write!(f, "invalid value name: {value} for property {property}")
            }
            Error::DuplicatedProperty(prop) => write!(f, "duplicated property: {prop}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "serde")]
mod _serde {
    use rimecraft_serde_update::Update;
    use serde::{Serialize, ser::SerializeMap};

    use crate::State;

    impl<T> Serialize for State<'_, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(Some(self.entries.len()))?;
            for (prop, val) in &self.entries {
                map.serialize_entry(
                    prop.name,
                    &prop.wrap.erased_to_name(*val).ok_or_else(|| {
                        serde::ser::Error::custom(format!(
                            "invalid value {val} in property {}",
                            prop.name
                        ))
                    })?,
                )?;
            }
            map.end()
        }
    }

    impl<'de, T> Update<'de> for &State<'_, T> {
        #[inline]
        fn update<D>(
            &mut self,
            deserializer: D,
        ) -> Result<(), <D as serde::Deserializer<'de>>::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<'a, T>(*const State<'a, T>);

            impl<'de, 'a, T> serde::de::Visitor<'de> for Visitor<'a, T> {
                type Value = *const State<'a, T>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a map")
                }

                fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    while let Some((prop, val)) = map.next_entry::<String, isize>()? {
                        self.0 = unsafe { &*self.0 }
                            .table
                            .get()
                            .expect("state not initialized")
                            .get(prop.as_str())
                            .ok_or_else(|| {
                                serde::de::Error::custom(format!("property {prop} not found"))
                            })?
                            .get(&val)
                            .ok_or_else(|| {
                                serde::de::Error::custom(format!(
                                    "value {val} not found in property {prop}"
                                ))
                            })?
                            .as_ptr()
                            .cast_const();
                    }
                    Ok(self.0)
                }
            }

            deserializer
                .deserialize_map(Visitor(*self))
                .map(|state| *self = unsafe { &*state })
        }
    }
}

unsafe impl<T: Send> Send for State<'_, T> {}
unsafe impl<T: Sync> Sync for State<'_, T> {}

unsafe impl<T: Send> Send for States<'_, T> {}
unsafe impl<T: Sync> Sync for States<'_, T> {}

#[cfg(test)]
mod tests;
