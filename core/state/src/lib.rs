//! Minecraft state holders.
//!
//! This corresponds to `net.minecraft.state` in `yarn`.

use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
    sync::{Arc, OnceLock, Weak},
};

use property::{BiIndex, ErasedProperty, Property, Wrap};
use regex_lite::Regex;

use crate::property::ErasedWrap;

pub mod property;

// <property> -> <<value> -> <state>>
type Table<'a, T> = HashMap<ErasedProperty<'a>, HashMap<isize, Weak<T>>>;

/// State of an object.
pub struct State<'a, T> {
    pub(crate) entries: Arc<HashMap<ErasedProperty<'a>, isize>>,
    table: OnceLock<Table<'a, Self>>,

    data: T,
}

impl<T> State<'_, T> {
    /// Gets the current value of given property in this state.
    #[inline]
    pub fn get<V, W>(&self, prop: &Property<'_, W>) -> Option<V>
    where
        W: Wrap<V> + BiIndex<V> + Hash + Eq + Send + Sync + 'static,
        for<'w> &'w W: IntoIterator<Item = V>,
    {
        self.entries
            .get(&ErasedProperty::from(prop))
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
    pub fn cycle<V, W>(&self, prop: &Property<'_, W>) -> Result<MaybeArc<'_, Self>, Error>
    where
        W: Wrap<V> + BiIndex<V> + Hash + Eq + Send + Sync + 'static,
        for<'w> &'w W: IntoIterator<Item = V>,
    {
        let erased = ErasedProperty::from(prop);

        let index = *self
            .entries
            .get(&erased)
            .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))?;
        let Some(next) = obtain_next(
            index,
            (&prop.wrap)
                .into_iter()
                .filter_map(|value| prop.wrap.index_of(&value)),
        ) else {
            return Ok(MaybeArc::Borrowed(self));
        };
        if next == index {
            Ok(MaybeArc::Borrowed(self))
        } else {
            self.table
                .get()
                .expect("state not initialized")
                .get(&erased)
                .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))
                .and_then(|map| map.get(&next).ok_or(Error::ValueNotFound(index)))
                .map(|weak| MaybeArc::Arc(weak.upgrade().expect("state was dropped")))
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
    pub fn with<V, W>(&self, prop: &Property<'_, W>, value: V) -> Result<MaybeArc<'_, Self>, Error>
    where
        W: Wrap<V> + BiIndex<V> + Hash + Eq + Send + Sync + 'static,
        for<'w> &'w W: IntoIterator<Item = V>,
    {
        let erased = ErasedProperty::from(prop);

        let index = *self
            .entries
            .get(&erased)
            .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))?;
        let value = prop.wrap.index_of(&value).ok_or(Error::InvalidValue)?;
        if value == index {
            Ok(MaybeArc::Borrowed(self))
        } else {
            self.table
                .get()
                .expect("state not initialized")
                .get(&erased)
                .ok_or_else(|| Error::PropertyNotFound(prop.name().to_string()))
                .and_then(|map| map.get(&value).ok_or(Error::ValueNotFound(index)))
                .map(|weak| MaybeArc::Arc(weak.upgrade().expect("state was dropped")))
        }
    }

    /// Whether this state contains given property.
    #[inline]
    pub fn contains<W, V>(&self, prop: &Property<'_, W>) -> bool
    where
        W: Wrap<V> + BiIndex<V> + Hash + Eq + Send + Sync + 'static,
        for<'w> &'w W: IntoIterator<Item = V>,
    {
        self.entries.contains_key(&ErasedProperty::from(prop))
    }

    /// Gets external data of this state.
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
    states: Vec<Arc<State<'a, T>>>,
    #[allow(unused)]
    props: BTreeMap<&'a str, ErasedProperty<'a>>,

    data: T,
}

impl<'a, T> States<'a, T> {
    fn new<I>(data: T, props: I) -> Self
    where
        T: Clone,
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
            .map(|vec| vec.into_iter().collect::<HashMap<_, _>>())
            .map(Arc::new)
            .map(|entries| {
                Arc::new(State {
                    entries: entries.clone(),
                    table: OnceLock::new(),
                    data: data.clone(),
                })
            })
            .collect::<Vec<_>>();

        // Initialize tables
        for state in list.iter() {
            let mut table: Table<'a, State<'a, T>> = Table::new();
            for (prop, s_val) in state.entries.iter() {
                let mut row = HashMap::new();
                for val in prop.wrap.erased_iter().filter(|v| v != s_val) {
                    let Some(s) = list.iter().find(|s| {
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
                    row.insert(val, Arc::downgrade(s));
                }
                table.insert(prop.clone(), row);
            }
            state.table.set(table).expect("state already initialized");
        }

        Self {
            states: list,
            props,
            data,
        }
    }

    /// Gets all states of this state.
    #[inline]
    pub fn states(&self) -> &[Arc<State<'a, T>>] {
        &self.states
    }

    /// Gets the external data.
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Gets the default state.
    ///
    /// # Panics
    ///
    /// Panics if no state is available.
    #[inline]
    pub fn default_state(&self) -> &Arc<State<'a, T>> {
        self.states.first().expect("no state available")
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

/// Mutable instance of [`States`].
#[derive(Debug)]
pub struct StatesMut<'a, T> {
    data: T,
    props: Vec<ErasedProperty<'a>>,
}

impl<'a, T> StatesMut<'a, T> {
    /// Creates a new states with given data.
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            data,
            props: Vec::new(),
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
    pub fn add<W, G>(&mut self, prop: &'a Property<'_, W>) -> Result<(), Error>
    where
        W: Wrap<G> + BiIndex<G> + Hash + Eq + Send + Sync + 'static,
        for<'w> &'w W: IntoIterator<Item = G>,
    {
        static NAME_PAT: OnceLock<Regex> = OnceLock::new();
        let reg = NAME_PAT.get_or_init(|| Regex::new(r"^[a-z0-9_]+$").unwrap());
        if !reg.is_match(prop.name()) {
            return Err(Error::InvalidPropertyName(prop.name().to_owned()));
        }
        let mut len = 0;
        for val in prop.wrap.erased_iter() {
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
        States::new(self.data, self.props)
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
            Error::PropertyNotFound(prop) => write!(f, "property not found: {}", prop),
            Error::TableNotPresent => write!(f, "table not present"),
            Error::ValueNotFound(value) => write!(f, "value not found: {}", value),
            Error::InvalidValue => write!(f, "invalid value"),
            Error::InvalidPropertyName(name) => write!(f, "invalid property name: {}", name),
            Error::PropertyContainsOneOrNoValue(prop) => {
                write!(f, "property {prop} contains <= 1 possible values")
            }
            Error::InvalidValueName { property, value } => {
                write!(f, "invalid value name: {value} for property {property}")
            }
            Error::DuplicatedProperty(prop) => write!(f, "duplicated property: {}", prop),
        }
    }
}

impl std::error::Error for Error {}

/// Cell that can be either an [`Arc`] or a borrowed reference.
#[derive(Debug)]
pub enum MaybeArc<'a, T> {
    /// The reference-counted variant.
    Arc(Arc<T>),
    /// The borrowed variant.
    Borrowed(&'a T),
}

impl<T> Hash for MaybeArc<'_, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Borrow::<T>::borrow(self).hash(state)
    }
}

impl<T> Borrow<T> for MaybeArc<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        match self {
            MaybeArc::Arc(arc) => arc.as_ref(),
            MaybeArc::Borrowed(val) => val,
        }
    }
}

impl<T> Clone for MaybeArc<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            MaybeArc::Arc(arc) => MaybeArc::Arc(Arc::clone(arc)),
            MaybeArc::Borrowed(val) => MaybeArc::Borrowed(val),
        }
    }
}

impl<T> PartialEq for MaybeArc<'_, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Borrow::<T>::borrow(self).eq(Borrow::<T>::borrow(other))
    }
}

impl<T> Eq for MaybeArc<'_, T> where T: Eq {}

impl<T> AsRef<T> for MaybeArc<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        Borrow::<T>::borrow(self)
    }
}

impl<T> Deref for MaybeArc<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Borrow::<T>::borrow(self)
    }
}

#[cfg(test)]
mod tests;
