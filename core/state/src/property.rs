//! State property types and traits.

use std::{any::TypeId, borrow::Cow, fmt::Debug, hash::Hash, ops::RangeInclusive};

#[derive(Clone)]
pub(crate) struct ErasedProperty<'a> {
    pub name: &'a str,
    pub ty: TypeId,
    pub wrap: &'a (dyn ErasedWrap + Send + Sync),

    wrap_hash: u64,
}

impl Debug for ErasedProperty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedProperty")
            .field("name", &self.name)
            .field("ty", &self.ty)
            .finish()
    }
}

impl Hash for ErasedProperty<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.ty.hash(state);
        self.wrap_hash.hash(state);
    }
}

impl PartialEq for ErasedProperty<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty && self.wrap_hash == other.wrap_hash
    }
}

impl Eq for ErasedProperty<'_> {}

/// Property of a state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Property<'a, W> {
    name: Cow<'a, str>,
    pub(crate) wrap: W,
}

impl<'a, W> Property<'a, W> {
    /// Creates a new property.
    #[inline]
    pub const fn new(name: &'a str, wrap: W) -> Self {
        Self {
            name: Cow::Borrowed(name),
            wrap,
        }
    }

    /// Creates a new property with an owned name.
    #[inline]
    pub fn with_owned_name(name: String, wrap: W) -> Self {
        Self {
            name: Cow::Owned(name),
            wrap,
        }
    }

    /// Returns the name of the property.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a, 'p, W> From<&'a Property<'p, W>> for ErasedProperty<'a>
where
    W: ErasedWrap + Hash + Send + Sync + 'static,
{
    fn from(prop: &'a Property<'p, W>) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        prop.wrap.hash(&mut hasher);

        ErasedProperty {
            name: &prop.name,
            ty: TypeId::of::<W>(),
            wrap: &prop.wrap,
            wrap_hash: std::hash::Hasher::finish(&hasher),
        }
    }
}

/// Property generic over the wrapped type.
pub trait Wrap<T> {
    /// Parses the name to the wrapped type.
    fn parse_name(&self, name: &str) -> Option<T>;

    /// Converts the wrapped type to a name.
    fn to_name<'a>(&'a self, value: &T) -> Option<Cow<'a, str>>;

    /// Returns the number of variants.
    fn variants(&self) -> usize;
}

pub(crate) trait ErasedWrap {
    fn erased_parse_name(&self, name: &str) -> Option<isize>;
    fn erased_to_name(&self, index: isize) -> Option<Cow<'_, str>>;
    fn erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = isize> + 'a>;
}

impl<T, G> ErasedWrap for T
where
    T: Wrap<G> + BiIndex<G>,
    for<'a> &'a T: IntoIterator<Item = G>,
{
    #[inline]
    fn erased_parse_name(&self, name: &str) -> Option<isize> {
        self.parse_name(name)
            .and_then(|value| self.index_of(&value))
    }

    #[inline]
    fn erased_to_name(&self, index: isize) -> Option<Cow<'_, str>> {
        BiIndex::index(self, index).and_then(|val| self.to_name(&val))
    }

    #[inline]
    fn erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = isize> + 'a> {
        struct Iter<I>
        where
            I: IntoIterator,
        {
            iter: <I as IntoIterator>::IntoIter,
            wrap: I,
        }

        impl<I> Iterator for Iter<I>
        where
            I: IntoIterator + BiIndex<<I as IntoIterator>::Item>,
        {
            type Item = isize;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter
                    .next()
                    .map(|val| self.wrap.index_of(&val).unwrap())
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        Box::new(Iter {
            iter: self.into_iter(),
            wrap: self,
        })
    }
}

/// Type wraps `T` with [`isize`].
pub trait BiIndex<T> {
    /// Converts the index to the wrapped type.
    fn index(&self, index: isize) -> Option<T>;

    /// Converts the wrapped type to an index.
    fn index_of(&self, value: &T) -> Option<isize>;
}

impl<T, I> BiIndex<I> for &T
where
    T: BiIndex<I>,
{
    #[inline]
    fn index(&self, index: isize) -> Option<I> {
        T::index(*self, index)
    }

    #[inline]
    fn index_of(&self, value: &I) -> Option<isize> {
        T::index_of(*self, value)
    }
}

mod bool;
mod int;

pub mod data {
    //! Property data types.

    pub use super::{bool::Data as BoolData, int::Data as IntData};
}

/// Property that has integer values.
#[doc(alias = "IntegerProperty")]
pub type IntProperty<'a, T = RangeInclusive<i32>> = Property<'a, int::Data<T>>;
/// Property that has boolean values.
#[doc(alias = "BooleanProperty")]
pub type BoolProperty<'a> = Property<'a, bool::Data>;
