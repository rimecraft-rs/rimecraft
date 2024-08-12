//! Minecraft chunk palette implementation.

use std::hash::Hash;

pub mod container;
mod iter;

use ahash::AHashMap;
pub use iter::Iter;
use iter::IterImpl;

pub use rimecraft_maybe::{Maybe, SimpleOwned};

/// A palette maps object from and to small integer IDs that uses less number of bits
/// to make storage smaller.
///
/// See [`Strategy`] for the available strategies.
#[derive(Debug, Clone)]
pub struct Palette<L, T> {
    list: L,
    index_bits: u32,
    internal: PaletteImpl<T>,
}

/// The strategy to use for the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive] // New strategies may be added in the future.
pub enum Strategy {
    /// A palette that only holds a unique entry.
    #[doc(alias = "SingleValue")]
    Singular,
    /// A palette that stores the possible entries in an array
    /// and maps them to their indices in the array.
    #[doc(alias = "Linear")]
    Array,
    /// A palette backed by bidirectional hash tables.
    #[doc(alias = "HashMap")]
    BiMap,
    /// A palette that directly maps the entries to their raw ID.
    #[doc(alias = "IdList")]
    #[doc(alias = "Global")]
    Direct,
}

/// The palette implementations.
#[derive(Debug, Clone)]
enum PaletteImpl<T> {
    /// A palette that only holds a unique entry.
    Singular(Option<T>),
    /// A palette that stores the possible entries in an array
    /// and maps them to their indices in the array.
    Array(Vec<T>),
    /// A palette backed by bidirectional hash tables.
    BiMap {
        forward: Vec<T>,
        reverse: AHashMap<T, usize>,
    },
    Direct,
}

impl<L, T> Palette<L, T>
where
    T: Clone + Hash + Eq,
{
    /// Creates a new palette with the given strategy and list of entries.
    ///
    /// # Panics
    ///
    /// ## Singular
    ///
    /// - Panics if the `bits_size` is not 0.
    /// - Panics if the `entries` length is greater than 1.
    ///
    /// ## Array and BiMap
    ///
    /// Panics if the `entries` length is greater than `2 ** bits_size`.
    pub fn new(strategy: Strategy, bits_size: u32, list: L, entries: Vec<T>) -> Self {
        match strategy {
            Strategy::Singular => {
                debug_assert_eq!(
                    bits_size, 0,
                    "illegal index bits for SingularPalette: {}",
                    bits_size
                );
                assert!(
                    entries.len() <= 1,
                    "can't initialize SingularPalette with {} entries",
                    entries.len()
                );

                Self {
                    list,
                    index_bits: 0,
                    internal: PaletteImpl::Singular(entries.into_iter().next()),
                }
            }
            Strategy::Array => {
                let mut array = Vec::with_capacity(1 << bits_size);
                assert!(
                    entries.len() <= array.capacity(),
                    "can't initialize ArrayPalette of length {} with {} entries",
                    array.len(),
                    entries.len()
                );
                array.extend(entries);

                Self {
                    list,
                    index_bits: bits_size,
                    internal: PaletteImpl::Array(array),
                }
            }
            Strategy::BiMap => {
                let len = 1 << bits_size;
                let mut forward = Vec::with_capacity(len);
                let mut reverse = AHashMap::with_capacity(len);
                assert!(
                    entries.len() <= forward.capacity(),
                    "can't initialize BiMapPalette of length {} with {} entries",
                    forward.len(),
                    entries.len()
                );
                reverse.extend(entries.iter().cloned().enumerate().map(|(i, v)| (v, i)));
                forward.extend(entries);

                Self {
                    list,
                    index_bits: bits_size,
                    internal: PaletteImpl::BiMap { forward, reverse },
                }
            }
            Strategy::Direct => Self {
                list,
                index_bits: 0,
                internal: PaletteImpl::Direct,
            },
        }
    }
}

impl<L, T> Palette<L, T>
where
    L: for<'a> IndexToRaw<&'a T>,
    T: Hash + Eq,
{
    /// Returns the ID of an object in the palette.
    pub fn index(&self, object: &T) -> Option<usize> {
        match &self.internal {
            PaletteImpl::Singular(value) => {
                value.as_ref().map_or(false, |v| v == object).then_some(0)
            }
            PaletteImpl::Array(array) => array.iter().position(|val| val == object),
            PaletteImpl::BiMap { reverse, .. } => reverse.get(object).copied(),
            PaletteImpl::Direct => Some(self.list.raw_id(object).unwrap_or(0)),
        }
    }

    /// Returns the ID of an object in the palette, or inserts it if absent.
    ///
    /// # Errors
    ///
    /// Returns `Err` containing the expected `index_bits` if the palette is too small to
    /// include this object.
    pub fn index_or_insert(&mut self, object: T) -> Result<usize, (u32, T)>
    where
        T: Clone,
    {
        match &mut self.internal {
            PaletteImpl::Singular(value) => {
                if let Some(entry) = value {
                    if entry != &object {
                        debug_assert_eq!(
                            self.index_bits, 0,
                            "illegal index bits for SingularPalette: {}",
                            self.index_bits
                        );
                        return Err((1, object));
                    }
                } else {
                    *value = Some(object);
                }
                Ok(0)
            }
            PaletteImpl::Array(array) => {
                if let Some(index) = array.iter().position(|val| val == &object) {
                    Ok(index)
                } else if array.capacity() > array.len() {
                    let index = array.len();
                    array.push(object);
                    Ok(index)
                } else {
                    Err((self.index_bits + 1, object))
                }
            }
            PaletteImpl::BiMap { forward, reverse } => {
                if let Some(index) = reverse.get(&object).copied() {
                    Ok(index)
                } else if forward.capacity() > forward.len() {
                    let index = forward.len();
                    forward.push(object.clone());
                    reverse.insert(object, index);
                    Ok(index)
                } else {
                    Err((self.index_bits + 1, object))
                }
            }
            PaletteImpl::Direct => self.index(&object).ok_or_else(|| unreachable!()),
        }
    }
}

impl<L, T> Palette<L, T>
where
    L: for<'s> IndexFromRaw<'s, Maybe<'s, T>>,
{
    /// Returns the object associated with the given ID.
    pub fn get(&self, id: usize) -> Option<Maybe<'_, T>> {
        match &self.internal {
            PaletteImpl::Singular(value) => (id == 0)
                .then_some(value.as_ref())
                .flatten()
                .map(Maybe::Borrowed),
            PaletteImpl::Array(forward) | PaletteImpl::BiMap { forward, .. } => {
                forward.get(id).map(Maybe::Borrowed)
            }
            PaletteImpl::Direct => self.list.of_raw(id),
        }
    }
}

impl<L, T> Palette<L, T> {
    /// Returns the size of this palette.
    #[allow(clippy::len_without_is_empty)] // A palette is never empty.
    pub fn len<'a>(&'a self) -> usize
    where
        &'a L: IntoIterator,
        <&'a L as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        match &self.internal {
            PaletteImpl::Singular(value) => value.is_some() as usize,
            PaletteImpl::Array(forward) | PaletteImpl::BiMap { forward, .. } => forward.len(),
            PaletteImpl::Direct => (&self.list).into_iter().len(),
        }
    }

    /// Returns an iterator over the palette.
    pub fn iter<'a, I>(&'a self) -> Iter<'_, I, T>
    where
        &'a L: IntoIterator<Item = &'a T, IntoIter = I>,
    {
        Iter {
            internal: match &self.internal {
                PaletteImpl::Singular(value) => IterImpl::MaybeNone(value.iter()),
                PaletteImpl::Array(forward) | PaletteImpl::BiMap { forward, .. } => {
                    IterImpl::Vector(forward.iter())
                }
                PaletteImpl::Direct => IterImpl::IntoIter((&self.list).into_iter()),
            },
        }
    }

    /// Returns the strategy and the bits size.
    #[inline]
    pub fn config(&self) -> (Strategy, u32) {
        (
            match self.internal {
                PaletteImpl::Singular(_) => Strategy::Singular,
                PaletteImpl::Array(_) => Strategy::Array,
                PaletteImpl::BiMap { .. } => Strategy::BiMap,
                PaletteImpl::Direct => Strategy::Direct,
            },
            self.index_bits,
        )
    }
}

impl<'a, L, T> IntoIterator for &'a Palette<L, T>
where
    &'a L: IntoIterator<Item = &'a T>,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, <&'a L as IntoIterator>::IntoIter, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "edcode")]
mod _edcode {

    use std::io::{self, ErrorKind};

    use edcode2::{Buf, BufExt, BufMut, BufMutExt, Decode, Encode};

    use super::*;

    impl<L, T, B> Encode<B> for Palette<L, T>
    where
        L: for<'a> IndexToRaw<&'a T>,
        B: BufMut,
    {
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            match &self.internal {
                PaletteImpl::Singular(value) => buf.put_variable(
                    value
                        .as_ref()
                        .ok_or(Error::Uninitialized)
                        .and_then(|v| self.list.raw_id(v).ok_or(Error::UnknownEntry))?
                        as u32,
                ),
                PaletteImpl::Array(forward) | PaletteImpl::BiMap { forward, .. } => {
                    buf.put_variable(forward.len() as u32);
                    for entry in forward {
                        buf.put_variable(self.list.raw_id(entry).ok_or(Error::UnknownEntry)? as u32)
                    }
                }
                PaletteImpl::Direct => {}
            }
            Ok(())
        }
    }

    impl<'de, L, T, B> Decode<'de, B> for Palette<L, T>
    where
        L: for<'s> IndexFromRaw<'s, T>,
        B: Buf,
    {
        fn decode_in_place(&mut self, mut buf: B) -> Result<(), edcode2::BoxedError<'de>> {
            match &mut self.internal {
                PaletteImpl::Singular(entry) => {
                    let id = buf.get_variable::<u32>() as usize;
                    *entry = Some(self.list.of_raw(id).ok_or_else(|| {
                        io::Error::new(ErrorKind::InvalidData, Error::UnknownId(id))
                    })?);
                }
                PaletteImpl::Array(forward) | PaletteImpl::BiMap { forward, .. } => {
                    let len = buf.get_variable::<u32>() as usize;
                    *forward = Vec::with_capacity(len);
                    for _ in 0..len {
                        let id = buf.get_variable::<u32>() as usize;
                        forward.push(self.list.of_raw(id).ok_or_else(|| {
                            io::Error::new(ErrorKind::InvalidData, Error::UnknownId(id))
                        })?);
                    }
                }
                PaletteImpl::Direct => {}
            }
            Ok(())
        }

        #[inline]
        fn decode(_buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            Err("palettes does not support non-in-place decoding".into())
        }

        const SUPPORT_NON_IN_PLACE: bool = false;
    }
}

/// A trait for types that can be indexed to raw ID.
pub trait IndexToRaw<T> {
    /// Returns the raw ID of the given entry.
    fn raw_id(&self, entry: T) -> Option<usize>;
}

/// A trait for types that can be indexed from raw ID.
pub trait IndexFromRaw<'s, T> {
    /// Returns the entry of the given raw ID.
    fn of_raw(&'s self, id: usize) -> Option<T>;
}

impl<T, S> IndexToRaw<T> for &S
where
    S: IndexToRaw<T>,
{
    #[inline]
    fn raw_id(&self, entry: T) -> Option<usize> {
        (**self).raw_id(entry)
    }
}

impl<'s, T, S> IndexFromRaw<'s, T> for &S
where
    S: IndexFromRaw<'s, T>,
{
    #[inline]
    fn of_raw(&'s self, id: usize) -> Option<T> {
        (**self).of_raw(id)
    }
}

/// Error type for palette operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The palette is uninitialized.
    Uninitialized,
    /// The entry is unable to be indexed into an ID.
    UnknownEntry,
    /// The raw ID is unknown.
    UnknownId(usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Uninitialized => write!(f, "use of an uninitialized palette"),
            Error::UnknownEntry => write!(f, "unknown entry"),
            Error::UnknownId(id) => write!(f, "unknown id: {}", id),
        }
    }
}

impl std::error::Error for Error {}
