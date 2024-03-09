//! Paletted containers.

use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use rimecraft_maybe::Maybe;
use rimecraft_packed_int_array::PackedIntArray;

use crate::{IndexFromRaw, IndexToRaw, Palette, Strategy};

/// A paletted container stores objects as small integer indices,
/// governed by palettes that map between these objects and indices.
#[derive(Debug)]
pub struct PalettedContainer<L, T, Cx> {
    list: L,
    data: Data<L, T>,
    _marker: PhantomData<Cx>,
}

impl<L, T, Cx> PalettedContainer<L, T, Cx>
where
    L: Clone,
    T: Clone + Hash + Eq,
{
    /// Creates a new paletted container with the given list of ids, palette
    /// configuration, storage and entries.
    pub fn new(list: L, config: (Strategy, u32), storage: Storage, entries: Vec<T>) -> Self {
        Self {
            data: Data {
                storage,
                palette: Palette::new(config.0, config.1, list.clone(), entries),
            },
            list,
            _marker: PhantomData,
        }
    }
}

macro_rules! resize {
    ($s:expr,$r:expr) => {
        match $r {
            Ok(e) => Some(e),
            Err(err) => $s.on_resize(err),
        }
    };
}

impl<L, T, Cx> PalettedContainer<L, T, Cx>
where
    L: for<'a> IndexToRaw<&'a T> + for<'s> IndexFromRaw<'s, Maybe<'s, T>> + Clone,
    T: Hash + Eq + Clone,
    Cx: ProvidePalette<L, T>,
{
    /// Creates a new paletted container that contains the given object.
    #[allow(clippy::missing_panics_doc)] // The panic point should be unreachable.
    pub fn of_single(list: L, object: T) -> Self {
        let data = compatible_data::<L, T, Cx>(list.clone(), None, 0)
            .expect("should return Some when prev is None");
        let mut this = Self {
            list,
            data,
            _marker: PhantomData,
        };
        resize!(this, this.data.palette.index_or_insert(object));
        this
    }

    /// Sets the value at the given index and returns the old one.
    pub fn swap(&mut self, index: usize, value: T) -> Option<Maybe<'_, T>> {
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

    /// Slices this container to a container of the first entry of the palette.
    ///
    /// See [`Self::of_single`].
    ///
    /// # Panics
    ///
    /// Panics if the palette is empty.
    pub fn to_slice(&self) -> Self {
        Self::of_single(
            self.list.clone(),
            self.data
                .palette
                .get(0)
                .expect("Palette should not be empty")
                .clone(),
        )
    }
}

impl<L, T, Cx> PalettedContainer<L, T, Cx>
where
    L: for<'s> IndexFromRaw<'s, Maybe<'s, T>>,
{
    /// Returns the value at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<Maybe<'_, T>> {
        self.data
            .storage
            .as_array()
            .and_then(|array| array.get(index))
            .and_then(|i| self.data.palette.get(i as usize))
    }

    /// Counts the number of occurrences of each object in the container
    /// to the given counter function.
    pub fn count<'a, F>(&'a self, mut counter: F)
    where
        F: FnMut(&T, usize),
        &'a L: IntoIterator,
        <&'a L as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        if let Some(val) = (self.data.palette.len() == 1)
            .then(|| self.data.palette.get(0))
            .flatten()
        {
            counter(&val, self.data.storage.len());
        } else {
            let mut map = HashMap::new();
            if let Some(array) = self.data.storage.as_array() {
                array.iter().for_each(|i| {
                    if let Some(val) = map.get_mut(&i) {
                        *val += 1;
                    } else {
                        map.insert(i, 1);
                    }
                });
            } else {
                map.insert(0, self.data.storage.len());
            }
            for (obj, c) in map
                .into_iter()
                .filter_map(|(i, c)| self.data.palette.get(i as usize).map(|i| (i, c)))
            {
                counter(&obj, c);
            }
        }
    }
}

fn compatible_data<L, T, Cx>(list: L, prev: Option<&Data<L, T>>, bits: u32) -> Option<Data<L, T>>
where
    T: Clone + Hash + Eq,
    Cx: ProvidePalette<L, T>,
{
    let config = Cx::provide_palette_config(&list, bits);
    if prev.is_some_and(|prev| prev.palette.config() == config) {
        None
    } else {
        Some(create_data(config, list, Cx::container_len()))
    }
}

impl<L, T, Cx> PalettedContainer<L, T, Cx>
where
    L: Clone + for<'a> IndexToRaw<&'a T> + for<'s> IndexFromRaw<'s, Maybe<'s, T>>,
    T: Clone + Hash + Eq,
    Cx: ProvidePalette<L, T>,
{
    fn on_resize(&mut self, (i, object): (u32, T)) -> Option<usize> {
        if let Some(mut data) = compatible_data::<L, T, Cx>(self.list.clone(), Some(&self.data), i)
        {
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
    const EDGE_BITS: u32;

    /// Returns the [`Strategy`] and the desired **entry length in bits**
    /// from the **given id list** and **given number of bits** needed to represent
    /// all palette entries.
    fn provide_palette_config(list: &L, bits: u32) -> (Strategy, u32);

    /// Returns the length of the container's data desired by this provider.
    #[inline]
    fn container_len() -> usize {
        1 << (Self::EDGE_BITS * 3)
    }

    /// Returns bits needed to represent the given list.
    fn bits(list: &L, len: usize) -> u32 {
        let i = usize::BITS - len.leading_zeros(); // ceil_log2
        let config = Self::provide_palette_config(list, i);
        if config.0 == Strategy::Direct {
            i
        } else {
            config.1
        }
    }
}

#[derive(Debug, Clone)]
struct Data<L, T> {
    storage: Storage,
    palette: Palette<L, T>,
}

/// A storage for paletted containers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum Storage {
    /// A packed array.
    PackedArray(PackedIntArray),
    /// An empty storage with length.
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

impl From<PackedIntArray> for Storage {
    #[inline]
    fn from(value: PackedIntArray) -> Self {
        Storage::PackedArray(value)
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
        L1: for<'s> IndexFromRaw<'s, Maybe<'s, T>>,
    {
        for i in 0..storage.len() {
            if let Some(raw) = storage
                .as_array()
                .and_then(|array| array.get(i))
                .and_then(|i| palette.get(i as usize))
                .and_then(|obj| self.palette.index(&obj))
            {
                self.storage.as_array_mut().unwrap().swap(i, raw as u32);
            }
        }
    }
}

#[inline]
fn create_data<L, T>((strategy, bits): (Strategy, u32), list: L, len: usize) -> Data<L, T>
where
    T: Clone + Hash + Eq,
{
    Data {
        storage: if bits == 0 {
            Storage::Empty(len)
        } else {
            Storage::PackedArray(
                PackedIntArray::from_packed(bits, len, None)
                    .expect("failed to create PackedIntArray"),
            )
        },
        palette: Palette::new(strategy, bits, list, vec![]),
    }
}

#[cfg(feature = "edcode")]
mod _edcode {
    use rimecraft_edcode::{Encode, Update, VarI32};

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

    impl<L, T, Cx> Encode for PalettedContainer<L, T, Cx>
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

    impl<L, T, Cx> Update for PalettedContainer<L, T, Cx>
    where
        L: for<'s> IndexFromRaw<'s, T> + Clone,
        T: Clone + Hash + Eq,
        Cx: ProvidePalette<L, T>,
    {
        fn update<B>(&mut self, mut buf: B) -> Result<(), std::io::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let data = compatible_data::<L, T, Cx>(
                self.list.clone(),
                Some(&self.data),
                buf.get_u8() as u32,
            );
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

    impl<L, T, Cx> PalettedContainer<L, T, Cx>
    where
        L: for<'a> IndexToRaw<&'a T>,
    {
        /// Returns the encoded length of this container.
        ///
        /// # Panics
        ///
        /// See errors in [`Palette::encoded_len`].
        #[inline]
        pub fn encoded_len(&self) -> usize {
            self.data.encoded_len()
        }
    }
}

#[cfg(feature = "serde")]
mod _serde {
    use crate::PaletteImpl;

    use super::*;

    use rimecraft_maybe::SimpleOwned;
    use rimecraft_serde_update::Update;
    use serde::{Deserialize, Serialize};

    fn write_palette_indices(this: &PackedIntArray, out: &mut [u32]) {
        let mut j = 0;

        for mut l in this.data()[..this.data().len() - 1].iter().copied() {
            for i in &mut out[j..j + this.elements_per_long()] {
                *i = (l & this.max()) as u32;
                l >>= this.element_bits();
            }
            j += this.elements_per_long();
        }

        if this.len() > j {
            let k = this.len() - j;
            let mut l = this.data()[this.data().len() - 1];
            for i in &mut out[j..j + k] {
                *i = (l & this.max()) as u32;
                l >>= this.element_bits();
            }
        }
    }

    fn apply_each<F>(is: &mut [u32], mut applier: F)
    where
        F: FnMut(u32) -> u32,
    {
        let mut i = -1i32 as u32;
        let mut j = -1i32 as u32;
        for l in is {
            let ll = *l;
            if ll != i {
                i = ll;
                j = applier(ll);
            }
            *l = j;
        }
    }

    impl<L, T, Cx> Serialize for PalettedContainer<L, T, Cx>
    where
        L: Clone + for<'a> IndexToRaw<&'a T> + for<'s> IndexFromRaw<'s, Maybe<'s, T>>,
        for<'a> &'a L: IntoIterator,
        for<'a> <&'a L as IntoIterator>::IntoIter: ExactSizeIterator,
        T: Clone + Hash + Eq + Serialize,
        Cx: ProvidePalette<L, T>,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut pal: Palette<L, T> = Palette::new(
                Strategy::BiMap,
                self.data
                    .storage
                    .as_array()
                    .map(PackedIntArray::element_bits)
                    .unwrap_or_default(),
                self.list.clone(),
                vec![],
            );

            let i = Cx::container_len();
            let mut is = vec![0; i];
            if let Some(array) = self.data.storage.as_array() {
                write_palette_indices(array, &mut is);
                // empty storages are always zeroed
            }
            apply_each(&mut is, |id| {
                pal.get(id as usize)
                    .map(|obj| match obj {
                        Maybe::Borrowed(obj) => obj.clone(),
                        Maybe::Owned(SimpleOwned(obj)) => obj,
                    })
                    .and_then(|obj| pal.index_or_insert(obj).ok())
                    .map_or(-1i32 as u32, |n| n as u32)
            });
            let pal = pal;
            let is = is;

            let j = Cx::bits(&self.list, pal.len());

            #[derive(Serialize)]
            struct Serialized<'a, T> {
                palette: &'a [T], // forward field in BiMapPalette
                data: Option<&'a [u64]>,
            }

            let entries = {
                let PaletteImpl::BiMap { forward, .. } = &pal.internal else {
                    unreachable!()
                };
                forward
            };
            if j != 0 {
                let arr = PackedIntArray::new(j, i, &is).expect("failed to create PackedIntArray");
                let ser = Serialized {
                    palette: entries,
                    data: Some(arr.data()),
                };
                ser.serialize(serializer)
            } else {
                let ser = Serialized {
                    palette: entries,
                    data: None,
                };
                ser.serialize(serializer)
            }
        }
    }

    impl<'de, L, T, Cx> Update<'de> for PalettedContainer<L, T, Cx>
    where
        L: Clone + for<'a> IndexToRaw<&'a T> + for<'s> IndexFromRaw<'s, Maybe<'s, T>>,
        T: Deserialize<'de> + Clone + Hash + Eq,
        Cx: ProvidePalette<L, T>,
    {
        fn update<D>(&mut self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct Serialized<T> {
                palette: Vec<T>,
                data: Option<Vec<u64>>,
            }
            let Serialized { palette, data } = Serialized::<T>::deserialize(deserializer)?;
            let i = Cx::container_len();
            let j = Cx::bits(&self.list, palette.len());
            let config = Cx::provide_palette_config(&self.list, j);
            let storage = if j != 0 {
                let ls = data.as_ref().ok_or_else(|| {
                    serde::de::Error::custom("missing values for non-zero storage")
                })?;
                Storage::PackedArray(
                    if config.0 == Strategy::Direct {
                        //FIXME: this is an expensive way. but it works
                        let pal = Palette::new(config.0, j, self.list.clone(), palette.clone());
                        let array = PackedIntArray::from_packed(j, i, Some(ls))
                            .map_err(serde::de::Error::custom)?;
                        let mut is = vec![0; i];
                        write_palette_indices(&array, &mut is);
                        apply_each(&mut is, |id| {
                            pal.get(id as usize)
                                .and_then(|obj| self.list.raw_id(&obj))
                                .map_or(-1i32 as u32, |n| n as u32)
                        });

                        PackedIntArray::new(config.1, i, &is)
                    } else {
                        PackedIntArray::from_packed(config.1, i, Some(ls))
                    }
                    .map_err(serde::de::Error::custom)?,
                )
            } else {
                Storage::Empty(i)
            };

            let list = self.list.clone();
            *self = Self::new(list, config, storage, palette);
            Ok(())
        }
    }
}
