use crate::network::{Decode, Encode};

/// A palette maps objects from and to small integer IDs that uses less
/// number of bits to make storage smaller.
///
/// While the objects palettes handle are already represented by integer
/// IDs, shrinking IDs in cases where only a few appear can further reduce
/// storage space and network traffic volume.
pub struct Palette<'a, T: 'a> {
    ids: crate::Ref<'a, dyn crate::collections::Indexed<T> + Send + Sync>,
    inner: Inner<'a, T>,
}

impl<'a, T: 'a> Palette<'a, T> {
    /// Returns the ID of an object in this palette.
    pub fn index(&self, value: &T) -> Option<usize> {
        match &self.inner {
            Inner::Vector(vec) => vec.iter().position(|option| {
                option.map_or(false, |e| {
                    e.0 as *const T as usize == value as *const T as usize
                })
            }),
            Inner::Indexed(ids) => ids.get_raw_id(value),
        }
    }

    /// Returns the ID of an object in this palette.
    /// If object does not yet exist in this palette, this palette will
    /// register the object.
    ///
    /// See [`Self::index`].
    pub fn index_or_insert(&mut self, value: &'a T) -> usize {
        self.index(value).unwrap_or_else(|| match &mut self.inner {
            Inner::Vector(vec) => {
                vec.push(Some(crate::Ref(value)));
                vec.len() - 1
            }
            Inner::Indexed(_) => 0,
        })
    }

    /// Returns `true` if any entry in this palette passes the predicate.
    pub fn any<P>(&self, predicate: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        match &self.inner {
            Inner::Vector(vec) => vec.iter().any(|e| e.map_or(false, |ee| predicate(ee.0))),
            Inner::Indexed(_) => true,
        }
    }

    /// Returns the object associated with the given `index`.
    pub fn get(&self, index: usize) -> Option<&'a T> {
        match &self.inner {
            Inner::Vector(vec) => vec.get(index).map(|e| e.map(|ee| ee.0)).flatten(),
            Inner::Indexed(ids) => ids.0.get(index),
        }
    }

    /// Initializes this palette from the `buf`.
    /// Clears the preexisting data in this palette.
    pub fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        match &mut self.inner {
            Inner::Vector(vec) => {
                vec.clear();

                for _ in 0..crate::VarInt::decode(buf)? {
                    vec.push(
                        self.ids
                            .0
                            .get(crate::VarInt::decode(buf)? as usize)
                            .map(|e| crate::Ref(e)),
                    );
                }
            }
            Inner::Indexed(_) => (),
        }

        Ok(())
    }

    /// Writes this palette to the `buf`.
    ///
    /// This is equal with [`Encode::encode`].
    pub fn write_buf<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        match &self.inner {
            Inner::Vector(vec) => {
                crate::VarInt(vec.len() as i32).encode(buf)?;

                for value in vec {
                    if let Some(r) = value {
                        crate::VarInt(
                            self.ids
                                .get_raw_id(r.0)
                                .map(|e| e as i32)
                                .unwrap_or(crate::collections::DEFAULT_INDEXED_INDEX),
                        )
                        .encode(buf)?;
                    } else {
                        crate::VarInt(crate::collections::DEFAULT_INDEXED_INDEX).encode(buf)?;
                    }
                }
            }
            Inner::Indexed(_) => (),
        }

        Ok(())
    }

    /// The serialized size of this palette in a byte buf, in bytes.
    pub fn buf_len(&self) -> usize {
        match &self.inner {
            Inner::Vector(vec) => {
                let mut i = crate::VarInt(vec.len() as i32).len() as usize;

                for value in vec {
                    if let Some(r) = value {
                        i += crate::VarInt(
                            self.ids
                                .get_raw_id(r.0)
                                .map(|e| e as i32)
                                .unwrap_or(crate::collections::DEFAULT_INDEXED_INDEX),
                        )
                        .len();
                    } else {
                        i += crate::VarInt(crate::collections::DEFAULT_INDEXED_INDEX).len();
                    }
                }

                i
            }
            Inner::Indexed(_) => crate::VarInt(0).len(),
        }
    }

    /// Size of this palette.
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::Vector(vec) => vec.len(),
            Inner::Indexed(ids) => ids.len(),
        }
    }
}

impl<'a, T: 'a> Encode for Palette<'a, T> {
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.write_buf(buf)
    }
}

impl<'a, T: 'a> Clone for Palette<'a, T> {
    fn clone(&self) -> Self {
        Self {
            ids: self.ids,
            inner: self.inner.clone(),
        }
    }
}

enum Inner<'a, T: 'a> {
    Indexed(crate::Ref<'a, dyn crate::collections::Indexed<T> + Send + Sync>),
    Vector(Vec<Option<crate::Ref<'a, T>>>),
    Singular(Option<crate::Ref<'a, T>>),
}

impl<'a, T: 'a> Clone for Inner<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Inner::Vector(vec) => Self::Vector(vec.clone()),
            Inner::Indexed(ids) => Self::Indexed(*ids),
            Inner::Singular(value) => Self::Singular(*value),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Indexed,
    Vector,
    Singular,
}

impl Variant {
    pub fn create<'a, A>(
        self,
        bits: usize,
        ids: &'a (dyn crate::collections::Indexed<A> + Send + Sync),
        entries: Vec<&'a A>,
    ) -> Palette<'a, A> {
        match self {
            Variant::Indexed => Palette {
                ids: crate::Ref(ids),
                inner: Inner::Indexed(crate::Ref(ids)),
            },
            Variant::Vector => Palette {
                ids: crate::Ref(ids),
                inner: Inner::Vector({
                    let mut vec = vec![None; 1 << bits];

                    for value in entries.into_iter().enumerate() {
                        vec[value.0] = Some(crate::Ref(value.1));
                    }

                    vec
                }),
            },
            Variant::Singular => Palette {
                ids: crate::Ref(ids),
                inner: Inner::Singular(entries.get(0).map(|e| crate::Ref(*e))),
            },
        }
    }
}

/// A storage whose values are raw IDs held by palettes.
#[derive(Clone)]
pub enum Storage {
    /// See [`crate::collections::PackedArray`].
    PackedArray(crate::collections::PackedArray),
    /// An empty palette storage has a length, but all its elements are 0.
    Empty(
        /// Length of this storage.
        usize,
    ),
}

impl Storage {
    /// Sets `value` to `index` and returns the previous value.
    pub fn swap(&mut self, index: usize, value: u64) -> u64 {
        match self {
            Storage::PackedArray(pa) => pa.swap(index, value),
            Storage::Empty(len) => {
                assert!(*len >= 1 && *len <= index + 1);
                0
            }
        }
    }

    /// Sets `value` to `index`.
    pub fn set(&mut self, index: usize, value: u64) {
        match self {
            Storage::PackedArray(pa) => pa.set(index, value),
            Storage::Empty(len) => {
                assert!(*len >= 1 && *len <= index + 1);
            }
        }
    }

    /// Returns the value at `index`.
    pub fn get(&self, index: usize) -> u64 {
        match self {
            Storage::PackedArray(pa) => pa.get(index),
            Storage::Empty(len) => {
                assert!(*len >= 1 && *len <= index + 1);
                0
            }
        }
    }

    /// The backing data of this storage.
    pub fn data(&self) -> &[u64] {
        match self {
            Storage::PackedArray(pa) => pa.data(),
            Storage::Empty(_) => &[],
        }
    }

    /// The length of, or the number of elements, in this storage.
    pub fn len(&self) -> usize {
        match self {
            Storage::PackedArray(pa) => pa.len(),
            Storage::Empty(len) => *len,
        }
    }

    /// The number of bits each element in this storage uses.
    pub fn element_bits(&self) -> usize {
        match self {
            Storage::PackedArray(pa) => pa.element_bits(),
            Storage::Empty(_) => 0,
        }
    }

    /// Executes an `action` on all values in this storage, sequentially.
    pub fn for_each<F>(&self, action: F)
    where
        F: Fn(u64),
    {
        match self {
            Storage::PackedArray(ap) => ap.for_each(action),
            Storage::Empty(len) => {
                for _ in 0..*len {
                    action(0)
                }
            }
        }
    }

    pub fn write_palette_indices(&self, out: &mut [i32]) {
        match self {
            Storage::PackedArray(pa) => pa.write_palette_indices(out),
            Storage::Empty(_) => out.fill(0),
        }
    }
}

struct Data<'a, T: 'a>(DataProvider, Storage, Palette<'a, T>);

#[derive(Clone, Copy)]
struct DataProvider(Variant, usize);

impl DataProvider {
    fn create_data<'a, T>(
        self,
        ids: &'a (dyn crate::collections::Indexed<T> + Send + Sync),
        len: usize,
    ) -> Data<'a, T> {
        Data(
            self,
            if self.1 == 0 {
                Storage::Empty(len)
            } else {
                Storage::PackedArray(crate::collections::PackedArray::new(self.1, len, None))
            },
            self.0.create(self.1, ids, Vec::new()),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    BlockState,
}

impl Provider {
    fn edge_bits(self) -> usize {
        match self {
            Provider::BlockState => 4,
        }
    }

    fn create_provider<A, I>(self, ids: &I, bits: usize) -> DataProvider
    where
        I: crate::collections::Indexed<A>,
    {
        match self {
            Provider::BlockState => match bits {
                0 => DataProvider(todo!(), bits),
                _ => todo!(),
            },
        }
    }
}
