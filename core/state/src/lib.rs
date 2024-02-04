//! Minecraft state holders.

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
    sync::Arc,
};

use property::{BiIndex, ErasedProperty, Property};

pub mod property;

/// State of an object.
pub struct State<'a, T> {
    pub(crate) entries: HashMap<ErasedProperty<'a>, isize>,
    // <property> -> <<value> -> <state>>
    table: Option<HashMap<ErasedProperty<'a>, HashMap<isize, Arc<Self>>>>,

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
    /// - Errors if the property `prop` is not present in this state.
    /// - Errors if the table is not present in this state.
    pub fn cycle<V, W>(&self, prop: &Property<'_, W>) -> Result<&Self, Error>
    where
        W: BiIndex<V>,
        for<'a> &'a W: IntoIterator<Item = V>,
    {
        let index = *self
            .entries
            .get(prop.name())
            .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))?;
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
                .as_ref()
                .ok_or(Error::TableNotPresent)
                .and_then(|table| {
                    table
                        .get(prop.name())
                        .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))
                })
                .and_then(|map| map.get(&next).ok_or(Error::ValueNotFound(index)))
                .map(Arc::as_ref)
        }
    }

    /// Gets the state of this state with given property `prop` set to `value`.
    ///
    /// # Errors
    ///
    /// - Errors if the property `prop` is not present in this state.
    /// - Errors if the value `value` is not present in the property `prop`.
    /// - Errors if the table is not present in this state.
    pub fn with<V, W>(&self, prop: &Property<'_, W>, value: &V) -> Result<&Self, Error>
    where
        W: BiIndex<V>,
    {
        let index = *self
            .entries
            .get(prop.name())
            .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))?;
        let value = prop.wrap.index_of(value).ok_or(Error::InvalidValue)?;
        if value == index {
            Ok(self)
        } else {
            self.table
                .as_ref()
                .ok_or(Error::TableNotPresent)
                .and_then(|table| {
                    table
                        .get(prop.name())
                        .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))
                })
                .and_then(|map| map.get(&value).ok_or(Error::ValueNotFound(index)))
                .map(Arc::as_ref)
        }
    }

    /// Whether this state contains given property.
    #[inline]
    pub fn contains<W>(&self, prop: &Property<'_, W>) -> bool {
        self.entries.contains_key(prop.name())
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

#[derive(Debug)]
#[doc(alias = "StateManager")]
pub struct States<'a, T> {
    states: Vec<Arc<State<'a, T>>>,
    props: BTreeMap<Cow<'a, str>, ErasedProperty<'a>>,
}

/// Error type for state operations.
#[derive(Debug)]
pub enum Error {
    /// The property was not found in the state.
    PropertyNotFound(String),
    /// The table was not found in the state.
    TableNotPresent,
    /// The value was not found in the property.
    ValueNotFound(isize),
    /// The value is invalid.
    InvalidValue,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PropertyNotFound(prop) => write!(f, "property not found: {}", prop),
            Error::TableNotPresent => write!(f, "table not present"),
            Error::ValueNotFound(value) => write!(f, "value not found: {}", value),
            Error::InvalidValue => write!(f, "invalid value"),
        }
    }
}

impl std::error::Error for Error {}
