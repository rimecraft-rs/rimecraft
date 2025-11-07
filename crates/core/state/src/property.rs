//! State property types and traits.

use std::{
    any::TypeId,
    borrow::{Borrow, Cow},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::RangeInclusive,
};

use smallbox::{SmallBox, smallbox};

#[derive(Clone)]
pub(crate) struct ErasedProperty<'a> {
    pub name: &'a str,
    pub ty: TypeId,
    pub wrap: &'a (dyn ErasedWrap + Send + Sync + 'a),
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
    }
}

impl Borrow<str> for ErasedProperty<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.name
    }
}

impl PartialEq for ErasedProperty<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty
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
    W: ErasedWrap + Send + Sync + 'p,
{
    #[inline]
    fn from(prop: &'a Property<'p, W>) -> Self {
        ErasedProperty {
            name: &prop.name,
            ty: typeid::of::<W>(),
            wrap: &prop.wrap,
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
    #[allow(dead_code)]
    fn erased_parse_name(&self, name: &str) -> Option<isize>;
    fn erased_to_name(&self, index: isize) -> Option<Cow<'_, str>>;
    fn erased_iter(&self) -> SmallBox<dyn Iterator<Item = isize> + '_, smallbox::space::S8>;
}

pub(crate) trait UnObjSafeErasedWrap {
    fn erased_iter_typed(&self) -> impl Iterator<Item = isize> + '_;
}

impl<T, G> UnObjSafeErasedWrap for T
where
    T: Wrap<G> + BiIndex<G>,
    for<'a> &'a T: IntoIterator<Item = G>,
{
    #[inline]
    fn erased_iter_typed(&self) -> impl Iterator<Item = isize> + '_ {
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

        Iter {
            iter: self.into_iter(),
            wrap: self,
        }
    }
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
    fn erased_iter(&self) -> SmallBox<dyn Iterator<Item = isize> + '_, smallbox::space::S8> {
        smallbox!(self.erased_iter_typed())
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

/// A property provider.
pub trait ProvideProperty<'p> {
    /// The property type.
    type Property;

    /// Provides a property.
    fn provide_property() -> &'p Self::Property;
}

/// A value with property provider.
#[derive(Debug)]
pub struct Value<T, Cx>(pub T, PhantomData<Cx>);

impl<T, Cx> Value<T, Cx> {
    /// Creates a new value.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use ::serde::{Deserialize, Serialize};

    use super::*;

    impl<'p, T, Cx> Serialize for Value<T, Cx>
    where
        Cx: ProvideProperty<'p> + 'p,
        <Cx as ProvideProperty<'p>>::Property: Wrap<T>,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let prop = Cx::provide_property();
            let name = prop
                .to_name(&self.0)
                .ok_or_else(|| ::serde::ser::Error::custom("value is not in the property"))?;
            name.serialize(serializer)
        }
    }

    impl<'p, 'de, T, Cx> Deserialize<'de> for Value<T, Cx>
    where
        Cx: ProvideProperty<'p> + 'p,
        <Cx as ProvideProperty<'p>>::Property: Wrap<T>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let prop = Cx::provide_property();
            let name = <Cow<'_, str>>::deserialize(deserializer)?;
            let value = prop
                .parse_name(&name)
                .ok_or_else(|| ::serde::de::Error::custom("invalid value"))?;
            Ok(Self::new(value))
        }
    }
}
