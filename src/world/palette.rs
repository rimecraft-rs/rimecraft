use crate::network::{Decode, Encode};

pub struct Palette<'a, T: 'a> {
    ids: crate::Ref<'a, dyn crate::collections::Indexed<T>>,
    inner: PaletteInner<'a, T>,
}

impl<'a, T: 'a> Palette<'a, T> {
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

    pub fn index_or_insert(&mut self, value: &'a T) -> usize {
        self.index(value).unwrap_or_else(|| match &mut self.inner {
            PaletteInner::Vector(vec) => {
                vec.push(Some(crate::Ref(value)));
                vec.len() - 1
            }
            PaletteInner::Indexed(_) => 0,
        })
    }

    pub fn any<P>(&self, p: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        match &self.inner {
            PaletteInner::Vector(vec) => vec.iter().any(|e| e.map_or(false, |ee| p(ee.0))),
            PaletteInner::Indexed(_) => true,
        }
    }

    pub fn get(&self, index: usize) -> Option<&'a T> {
        match &self.inner {
            PaletteInner::Vector(vec) => vec.get(index).map(|e| e.map(|ee| ee.0)).flatten(),
            PaletteInner::Indexed(ids) => ids.0.get(index),
        }
    }

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

enum PaletteInner<'a, T: 'a> {
    Vector(Vec<Option<crate::Ref<'a, T>>>),
    Indexed(crate::Ref<'a, dyn crate::collections::Indexed<T>>),
}

impl<'a, T: 'a> Clone for PaletteInner<'a, T> {
    fn clone(&self) -> Self {
        match self {
            PaletteInner::Vector(vec) => Self::Vector(vec.clone()),
            PaletteInner::Indexed(ids) => Self::Indexed(*ids),
        }
    }
}
