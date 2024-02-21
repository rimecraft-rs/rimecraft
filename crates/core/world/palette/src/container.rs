//! Paletted containers.

use std::{hash::Hash, marker::PhantomData};

use rimecraft_packed_int_array::PackedIntArray;

use crate::{IndexFromRaw, IndexToRaw, Palette, Strategy};

/// A paletted container stores objects as small integer indices,
/// governed by palettes that map between these objects and indices.
#[derive(Debug)]
pub struct PalettedContainer<L, T, P> {
    list: L,
    data: Data<L, T>,
    _marker: PhantomData<P>,
}

macro_rules! resize {
    ($s:expr,$r:expr) => {
        match $r {
            Ok(e) => Some(e),
            Err(err) => $s.on_resize(err),
        }
    };
}

impl<L, T, P> PalettedContainer<L, T, P>
where
    L: for<'a> IndexToRaw<&'a T> + for<'s> IndexFromRaw<'s, &'s T> + Clone,
    T: Hash + Eq + Clone,
    P: ProvidePalette<L, T>,
{
    /// Sets the value at the given index and returns the old one.
    pub fn swap(&mut self, index: usize, value: T) -> Option<&T> {
        resize!(self, self.data.palette.index_or_insert(value))
            .and_then(|i| {
                if let Some(array) = self.data.storage.as_array_mut() {
                    array.swap(index, i as u32)
                } else {
                    None
                }
            })
            .and_then(|i| self.data.palette.get(i as usize))
    }

    /// Returns the value at the given index.
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        if let (Some(i), Some(array)) = (
            resize!(self, self.data.palette.index_or_insert(value)),
            self.data.storage.as_array_mut(),
        ) {
            array.set(index, i as u32)
        }
    }
}

impl<L, T, P> PalettedContainer<L, T, P>
where
    L: for<'s> IndexFromRaw<'s, &'s T>,
{
    /// Returns the value at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data
            .storage
            .as_array()
            .and_then(|array| array.get(index))
            .and_then(|i| self.data.palette.get(i as usize))
    }
}

impl<L, T, P> PalettedContainer<L, T, P>
where
    L: Clone,
    T: Clone + Hash + Eq,
    P: ProvidePalette<L, T>,
{
    fn compatible_data(&self, prev: Option<&Data<L, T>>, bits: usize) -> Option<Data<L, T>> {
        let config = P::provide_palette_config(&self.list, bits);
        if prev.is_some_and(|prev| prev.palette.config() == config) {
            None
        } else {
            Some(create_data(
                config,
                self.list.clone(),
                1 << (P::EDGE_BITS * 3),
            ))
        }
    }

    fn on_resize(&mut self, (i, object): (usize, T)) -> Option<usize>
    where
        L: for<'a> IndexToRaw<&'a T> + for<'s> IndexFromRaw<'s, &'s T>,
    {
        if let Some(mut data) = self.compatible_data(Some(&self.data), i) {
            data.import_from(&self.data.palette, &self.data.storage);
            self.data = data;
            self.data.palette.index(&object)
        } else {
            None
        }
    }
}

/// Types determines what type of palette to choose given the bits used to
/// represent each element.
///
/// In addition, it controls how the data in the serialized container is read
/// based on the palette given.
pub trait ProvidePalette<L, T> {
    /// `edgeBits` field of this palette.
    const EDGE_BITS: usize;

    /// Returns the [`Strategy`] and the desired **entry length in bits**
    /// from the **given id list** and **given number of bits** needed to represent
    /// all palette entries.
    fn provide_palette_config(id_list: &L, bits: usize) -> (Strategy, usize);
}

#[derive(Debug, Clone)]
struct Data<L, T> {
    storage: Storage,
    palette: Palette<L, T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Storage {
    PackedArray(PackedIntArray),
    Empty(usize),
}

impl Storage {
    #[inline]
    fn as_array(&self) -> Option<&PackedIntArray> {
        match self {
            Storage::PackedArray(array) => Some(array),
            _ => None,
        }
    }

    #[inline]
    fn as_array_mut(&mut self) -> Option<&mut PackedIntArray> {
        match self {
            Storage::PackedArray(array) => Some(array),
            _ => None,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            Storage::PackedArray(array) => array.len(),
            Storage::Empty(len) => *len,
        }
    }
}

impl<L, T> Data<L, T>
where
    L: for<'a> IndexToRaw<&'a T>,
    T: Hash + Eq,
{
    /// Imports the data from the other palette and storage.
    #[allow(clippy::missing_panics_doc)]
    pub fn import_from<L1>(&mut self, palette: &Palette<L1, T>, storage: &Storage)
    where
        L1: for<'s> IndexFromRaw<'s, &'s T>,
    {
        for i in 0..storage.len() {
            if let Some(raw) = storage
                .as_array()
                .and_then(|array| array.get(i))
                .and_then(|i| palette.get(i as usize))
                .and_then(|obj| self.palette.index(obj))
            {
                self.storage.as_array_mut().unwrap().swap(i, raw as u32);
            }
        }
    }
}

#[inline]
fn create_data<L, T>((strategy, bits): (Strategy, usize), list: L, len: usize) -> Data<L, T>
where
    T: Clone + Hash + Eq,
{
    Data {
        storage: if bits == 0 {
            Storage::Empty(len)
        } else {
            Storage::PackedArray(PackedIntArray::from_packed(bits, len, None))
        },
        palette: Palette::new(strategy, bits, list, vec![]),
    }
}

#[cfg(feature = "edcode")]
mod _edcode {
    use rimecraft_edcode::{Encode, Update, VarI32};

    use crate::IndexToRaw;

    use super::*;

    impl<L, T> Encode for Data<L, T>
    where
        L: for<'a> IndexToRaw<&'a T>,
    {
        fn encode<B>(&self, mut buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            buf.put_u8(
                self.storage
                    .as_array()
                    .map(PackedIntArray::element_bits)
                    .unwrap_or_default() as u8,
            );
            self.palette.encode(&mut buf)?;
            self.storage
                .as_array()
                .map(PackedIntArray::data)
                .unwrap_or(&[])
                .encode(&mut buf)
        }
    }

    impl<L, T> Data<L, T>
    where
        L: for<'a> IndexToRaw<&'a T>,
    {
        /// Returns the encoded length of this data.
        ///
        /// # Panics
        ///
        /// See errors in [`Palette::encoded_len`].
        pub fn encoded_len(&self) -> usize {
            let len = self
                .storage
                .as_array()
                .map(|array| array.data().len())
                .unwrap_or_default();
            1 + self
                .palette
                .encoded_len()
                .expect("palette is not encodable")
                + VarI32(len as i32).encoded_len()
                + len * 8
        }
    }

    impl<L, T, P> Encode for PalettedContainer<L, T, P>
    where
        L: for<'a> IndexToRaw<&'a T>,
    {
        #[inline]
        fn encode<B>(&self, buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            self.data.encode(buf)
        }
    }

    impl<L, T, P> Update for PalettedContainer<L, T, P>
    where
        L: for<'s> IndexFromRaw<'s, T> + Clone,
        T: Clone + Hash + Eq,
        P: ProvidePalette<L, T>,
    {
        fn update<B>(&mut self, mut buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let data = self.compatible_data(Some(&self.data), buf.get_u8() as usize);
            if let Some(data) = data {
                self.data = data
            }

            self.data.palette.update(&mut buf)?;
            if let Some(array) = self.data.storage.as_array_mut() {
                array.data_mut().update(&mut buf)?;
            }

            Ok(())
        }
    }

    impl<L, T, P> PalettedContainer<L, T, P>
    where
        L: for<'a> IndexToRaw<&'a T>,
    {
        /// Returns the encoded length of this container.
        ///
        /// # Panics
        ///
        /// See errors in [`Data::encoded_len`].
        #[inline]
        pub fn encoded_len(&self) -> usize {
            self.data.encoded_len()
        }
    }
}
