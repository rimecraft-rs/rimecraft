use crate::network::{Decode, Encode};

/// A palette maps objects from and to small integer IDs that uses less
/// number of bits to make storage smaller.
///
/// While the objects palettes handle are already represented by integer
/// IDs, shrinking IDs in cases where only a few appear can further reduce
/// storage space and network traffic volume.
pub struct Palette<'a, T: 'a> {
    ids: crate::Ref<'a, dyn crate::collections::Indexed<T>>,
    inner: PaletteInner<'a, T>,
}

impl<'a, T: 'a> Palette<'a, T> {
    /// Returns the ID of an object in this palette.
    pub fn index(&self, value: &T) -> Option<usize> {
        match &self.inner {
            PaletteInner::Vector(vec) => vec.iter().position(|option| {
                option.map_or(false, |e| {
                    e.0 as *const T as usize == value as *const T as usize
                })
            }),
            PaletteInner::Indexed(ids) => ids.get_raw_id(value),
        }
    }

    /// Returns the ID of an object in this palette.
    /// If object does not yet exist in this palette, this palette will
    /// register the object.
    ///
    /// See [`Self::index`].
    pub fn index_or_insert(&mut self, value: &'a T) -> usize {
        self.index(value).unwrap_or_else(|| match &mut self.inner {
            PaletteInner::Vector(vec) => {
                vec.push(Some(crate::Ref(value)));
                vec.len() - 1
            }
            PaletteInner::Indexed(_) => 0,
        })
    }

    /// Returns `true` if any entry in this palette passes the predicate.
    pub fn any<P>(&self, predicate: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        match &self.inner {
            PaletteInner::Vector(vec) => vec.iter().any(|e| e.map_or(false, |ee| predicate(ee.0))),
            PaletteInner::Indexed(_) => true,
        }
    }

    /// Returns the object associated with the given `index`.
    pub fn get(&self, index: usize) -> Option<&'a T> {
        match &self.inner {
            PaletteInner::Vector(vec) => vec.get(index).map(|e| e.map(|ee| ee.0)).flatten(),
            PaletteInner::Indexed(ids) => ids.0.get(index),
        }
    }

    /// Initializes this palette from the `buf`.
    /// Clears the preexisting data in this palette.
    pub fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        match &mut self.inner {
            PaletteInner::Vector(vec) => {
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
            PaletteInner::Indexed(_) => (),
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
            PaletteInner::Vector(vec) => {
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
            PaletteInner::Indexed(_) => (),
        }

        Ok(())
    }

    /// The serialized size of this palette in a byte buf, in bytes.
    pub fn buf_len(&self) -> usize {
        match &self.inner {
            PaletteInner::Vector(vec) => {
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
            PaletteInner::Indexed(_) => crate::VarInt(0).len(),
        }
    }

    /// Size of this palette.
    pub fn len(&self) -> usize {
        match &self.inner {
            PaletteInner::Vector(vec) => vec.len(),
            PaletteInner::Indexed(ids) => ids.len(),
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

unsafe impl<'a, T: 'a> Send for Palette<'a, T> {}
unsafe impl<'a, T: 'a> Sync for Palette<'a, T> {}

enum PaletteInner<'a, T: 'a> {
    Indexed(crate::Ref<'a, dyn crate::collections::Indexed<T> + Send + Sync>),
    Vector(Vec<Option<crate::Ref<'a, T>>>),
}

impl<'a, T: 'a> Clone for PaletteInner<'a, T> {
    fn clone(&self) -> Self {
        match self {
            PaletteInner::Vector(vec) => Self::Vector(vec.clone()),
            PaletteInner::Indexed(ids) => Self::Indexed(*ids),
        }
    }
}
