use std::{convert::Infallible, hash::Hash};

use rimecraft_collections::PackedArray;
use rimecraft_edcode::{error::ErrorWithVarI32Err, Decode, Encode, VarI32};

pub mod error {
    #[derive(Debug)]
    pub struct RawIdNotFoundError;

    impl std::fmt::Display for RawIdNotFoundError {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "raw id not found in the target id list")
        }
    }

    impl std::error::Error for RawIdNotFoundError {}
}

use error::*;

/// A palette maps objects from and to small integer IDs that uses less
/// number of bits to make storage smaller.
///
/// While the objects palettes handle are already represented by integer
/// IDs, shrinking IDs in cases where only a few appear can further reduce
/// storage space and network traffic volume.
pub struct Palette<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    ids: rimecraft_primitives::Ref<'a, dyn rimecraft_collections::Index<T> + Send + Sync>,
    inner: Inner<'a, T>,
}

impl<'a, T> Palette<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    /// Returns the ID of an object in this palette.
    pub fn index(&self, value: T) -> Option<usize> {
        match &self.inner {
            Inner::Vector(vec) => vec.iter().position(|e| e == &value),
            Inner::Indexed(ids) => ids.index_of(&value),
            Inner::Singular(option) => option
                .map(|e| if e == value { Some(0) } else { None })
                .flatten(),
            Inner::BiMap(bimap) => bimap.get_by_right(&value).copied(),
        }
    }

    /// Returns the ID of an object in this palette.
    /// If object does not yet exist in this palette, this palette will
    /// register the object.
    ///
    /// See [`Self::index`].
    pub fn index_or_insert(&mut self, value: T) -> usize {
        self.index(value).unwrap_or_else(|| match &mut self.inner {
            Inner::Vector(vec) => {
                vec.push(value);
                vec.len() - 1
            }
            Inner::Indexed(_) => 0,
            Inner::Singular(option) => {
                *option = Some(value);
                0
            }
            Inner::BiMap(bimap) => bimap.get_by_right(&value).copied().unwrap_or_else(|| {
                let id = bimap.len();
                bimap.insert(id, value);
                id
            }),
        })
    }

    /// Returns `true` if any entry in this palette passes the predicate.
    pub fn any<P>(&self, predicate: P) -> bool
    where
        P: Fn(T) -> bool,
    {
        match &self.inner {
            Inner::Vector(vec) => vec.iter().any(|e| predicate(*e)),
            Inner::Indexed(_) => true,
            Inner::Singular(option) => predicate(option.expect("use of an uninitialized palette")),
            Inner::BiMap(bimap) => bimap.iter().any(|entry| predicate(*entry.1)),
        }
    }

    /// Returns the object associated with the given `index`.
    pub fn get(&self, index: usize) -> Option<T> {
        match &self.inner {
            Inner::Vector(vec) => vec.get(index).copied(),
            Inner::Indexed(ids) => ids.0.get(index).copied(),
            Inner::Singular(option) => {
                if let (Some(e), 0) = (option, index) {
                    Some(*e)
                } else {
                    panic!("missing palette entry for id {index}")
                }
            }
            Inner::BiMap(bimap) => {
                if let Some(value) = bimap.get_by_left(&index) {
                    Some(*value)
                } else {
                    panic!("missing palette entry for id {index}")
                }
            }
        }
    }

    /// The serialized size of this palette in a byte buf, in bytes.
    pub fn buf_len(&self) -> usize {
        match &self.inner {
            Inner::Vector(vec) => {
                VarI32(vec.len() as i32).len() as usize
                    + vec
                        .iter()
                        .map(|value| {
                            VarI32(self.ids.index_of(value).map(|e| e as i32).unwrap_or(-1)).len()
                        })
                        .sum::<usize>()
            }
            Inner::Indexed(_) => 0,
            Inner::Singular(option) => {
                if let Some(entry) = option {
                    VarI32(self.ids.index_of(&entry).map(|e| e as i32).unwrap_or(-1)).len()
                } else {
                    panic!("use of an uninitialized palette");
                }
            }
            Inner::BiMap(bimap) => {
                VarI32(self.len() as i32).len()
                    + bimap
                        .iter()
                        .map(|entry| {
                            VarI32(self.ids.index_of(entry.1).map(|e| e as i32).unwrap_or(-1)).len()
                        })
                        .sum::<usize>()
            }
        }
    }

    /// Size of this palette.
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::Vector(vec) => vec.len(),
            Inner::Indexed(ids) => ids.len(),
            Inner::Singular(_) => 1,
            Inner::BiMap(bimap) => bimap.len(),
        }
    }
}

impl<'a, T> Encode for Palette<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    type Error = Infallible;

    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        match &self.inner {
            Inner::Vector(vec) => {
                VarI32(vec.len() as i32).encode(&mut buf)?;

                for value in vec {
                    VarI32(self.ids.index_of(value).map(|e| e as i32).unwrap_or(-1))
                        .encode(&mut buf)?;
                }
            }
            Inner::Indexed(_) => (),
            Inner::Singular(option) => {
                if let Some(entry) = option {
                    VarI32(self.ids.index_of(&entry).map(|e| e as i32).unwrap_or(-1))
                        .encode(&mut buf)?;
                } else {
                    panic!("use of uninitialized palette");
                }
            }
            Inner::BiMap(bimap) => {
                let i = self.len();
                VarI32(i as i32).encode(&mut buf)?;

                for entry in bimap.iter() {
                    VarI32(self.ids.index_of(entry.1).map(|e| e as i32).unwrap_or(-1))
                        .encode(&mut buf)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a, T> rimecraft_edcode::Update for Palette<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    type Error = ErrorWithVarI32Err<RawIdNotFoundError>;

    fn update<B>(&mut self, mut buf: B) -> Result<(), <Self as rimecraft_edcode::Update>::Error>
    where
        B: bytes::Buf,
    {
        match &mut self.inner {
            Inner::Vector(vec) => {
                *vec = Vec::new();

                for _ in 0..VarI32::decode(&mut buf)? {
                    vec.push(*self.ids.0.get(VarI32::decode(&mut buf)? as usize).unwrap());
                }
            }
            Inner::Indexed(_) => (),
            Inner::Singular(option) => {
                *option = Some(
                    self.ids
                        .get(VarI32::decode(&mut buf)? as usize)
                        .copied()
                        .ok_or_else(|| ErrorWithVarI32Err::Target(RawIdNotFoundError))?,
                );
            }
            Inner::BiMap(bimap) => {
                let mut map = bimap::BiMap::new();
                for _ in 0..VarI32::decode(&mut buf)? {
                    map.insert(
                        map.len(),
                        *self.ids.0.get(VarI32::decode(&mut buf)? as usize).unwrap(),
                    );
                }
                *bimap = map
            }
        }

        Ok(())
    }
}

impl<'a, T> Clone for Palette<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    fn clone(&self) -> Self {
        Self {
            ids: self.ids,
            inner: self.inner.clone(),
        }
    }
}

enum Inner<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    Indexed(rimecraft_primitives::Ref<'a, dyn rimecraft_collections::Index<T> + Send + Sync>),
    Vector(Vec<T>),
    Singular(Option<T>),
    BiMap(bimap::BiMap<usize, T>),
}

impl<'a, T> Clone for Inner<'a, T>
where
    T: Eq + Copy + Hash + 'a,
{
    fn clone(&self) -> Self {
        match self {
            Inner::Indexed(ids) => Self::Indexed(*ids),
            Inner::Vector(vec) => Self::Vector(vec.clone()),
            Inner::Singular(value) => Self::Singular(*value),
            Inner::BiMap(bimap) => Self::BiMap(bimap.clone()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Indexed,
    Vector,
    Singular,
    BiMap,
}

impl Variant {
    pub fn create<'a, A>(
        self,
        bits: usize,
        ids: &'a (dyn rimecraft_collections::Index<A> + Send + Sync),
        entries: Vec<A>,
    ) -> Palette<'a, A>
    where
        A: Eq + Copy + Hash + 'a,
    {
        match self {
            Variant::Indexed => Palette {
                ids: rimecraft_primitives::Ref(ids),
                inner: Inner::Indexed(rimecraft_primitives::Ref(ids)),
            },
            Variant::Vector => Palette {
                ids: rimecraft_primitives::Ref(ids),
                inner: Inner::Vector({
                    let mut vec = Vec::with_capacity(1 << bits);
                    for value in entries.into_iter().enumerate() {
                        vec[value.0] = value.1;
                    }
                    vec
                }),
            },
            Variant::Singular => Palette {
                ids: rimecraft_primitives::Ref(ids),
                inner: Inner::Singular(entries.get(0).copied()),
            },
            Variant::BiMap => Palette {
                ids: rimecraft_primitives::Ref(ids),
                inner: Inner::BiMap({
                    let mut map = bimap::BiMap::with_capacity(1 << bits);
                    for entry in entries {
                        map.insert(map.len(), entry);
                    }
                    map
                }),
            },
        }
    }
}

pub type Storage = PackedArray;

struct Data<'a, T: 'a>(DataProvider, Storage, Palette<'a, T>)
where
    T: Eq + Copy + Hash;

#[derive(Clone, Copy, PartialEq, Eq)]
struct DataProvider(Variant, usize);

impl DataProvider {
    fn create_data<'a, T>(
        self,
        ids: &'a (dyn rimecraft_collections::Index<T> + Send + Sync),
        len: usize,
    ) -> Data<'a, T>
    where
        T: Eq + Copy + Hash,
    {
        Data(
            self,
            if self.1 == 0 {
                Storage::new(1, len, None)
            } else {
                Storage::new(self.1, len, None)
            },
            self.0.create(self.1, ids, Vec::new()),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    BlockState,
    Biome,
}

impl Provider {
    fn edge_bits(self) -> usize {
        match self {
            Provider::BlockState => 4,
            Provider::Biome => 2,
        }
    }

    fn create_provider<A>(
        self,
        ids: &(dyn rimecraft_collections::Index<A>),
        bits: usize,
    ) -> DataProvider {
        match self {
            Provider::BlockState => match bits {
                0 => DataProvider(Variant::Singular, bits),
                1 | 2 | 3 | 4 => DataProvider(Variant::Vector, 4),
                5 | 6 | 7 | 8 => DataProvider(Variant::BiMap, bits),
                _ => DataProvider(
                    Variant::Indexed,
                    crate::math::impl_helper::ceil_log_2(ids.len() as i32) as usize,
                ),
            },
            Provider::Biome => match bits {
                0 => DataProvider(Variant::Singular, bits),
                1 | 2 | 3 => DataProvider(Variant::Vector, bits),
                _ => DataProvider(
                    Variant::Indexed,
                    crate::math::impl_helper::ceil_log_2(ids.len() as i32) as usize,
                ),
            },
        }
    }

    pub fn compute_index(self, x: i32, y: i32, z: i32) -> usize {
        ((y << self.edge_bits() | z) << self.edge_bits() | x) as usize
    }

    pub fn container_size(self) -> usize {
        1 << (self.edge_bits() * 3)
    }
}

/// A paletted container stores objects in 3D voxels as small integer indices,
/// governed by "palettes" that map between these objects and indices.
pub struct Container<'a, T: 'a>(parking_lot::RwLock<ContainerInner<'a, T>>)
where
    T: Eq + Copy + Hash;

impl<'a, T: 'a> Container<'a, T>
where
    T: Eq + Copy + Hash,
{
    pub fn new(
        ids: &'a (dyn rimecraft_collections::Index<T> + Send + Sync),
        provider: Provider,
        variant: Variant,
        bits: usize,
        storage: Storage,
        entries: Vec<T>,
    ) -> Self {
        Self(parking_lot::RwLock::new(ContainerInner {
            ids: rimecraft_primitives::Ref(ids),
            data: Some(Data(
                DataProvider(variant, bits),
                storage,
                variant.create(bits, ids, entries),
            )),
            provider,
        }))
    }

    pub fn from_initialize(
        ids: &'a (dyn rimecraft_collections::Index<T> + Send + Sync),
        object: T,
        provider: Provider,
    ) -> Self {
        let mut this = ContainerInner {
            ids: rimecraft_primitives::Ref(ids),
            data: None,
            provider,
        };

        this.data = Some(this.get_compatible_data(None, 0));
        this.data.as_mut().unwrap().2.index_or_insert(object);

        Self(parking_lot::RwLock::new(this))
    }

    #[inline]
    pub fn swap(&self, pos: (i32, i32, i32), value: T) -> T {
        let _this = self.0.write();
        unsafe { self.swap_unchecked(pos, value) }
    }

    pub unsafe fn swap_unchecked(&self, (x, y, z): (i32, i32, i32), value: T) -> T {
        let this = self.0.data_ptr().as_mut().unwrap();
        let data = this.data.as_mut().unwrap();

        let i = data.2.index_or_insert(value);
        let j = data.1.swap(this.provider.compute_index(x, y, z), i as u64);

        data.2.get(j as usize).unwrap()
    }

    pub fn set(&self, (x, y, z): (i32, i32, i32), value: T) {
        let mut this = self.0.write();

        let index = this.provider.compute_index(x, y, z);
        let data = this.data.as_mut().unwrap();
        let i = data.2.index_or_insert(value);

        data.1.set(index, i as u64);
    }

    pub fn get(&self, (x, y, z): (i32, i32, i32)) -> Option<T> {
        let this = self.0.read();
        let data = this.data.as_ref().unwrap();

        let index = this.provider.compute_index(x, y, z);
        data.2.get(data.1.get(index) as usize)
    }

    pub fn for_each<F>(&self, action: F)
    where
        F: Fn(T),
    {
        let this = self.0.read();
        let data = this.data.as_ref().unwrap();

        let mut vec = Vec::new();
        let ptr = &mut vec as *mut Vec<u64>;

        data.1.iter().for_each(|value| {
            let v = unsafe { ptr.as_mut().unwrap() };

            if !v.contains(&value) {
                v.push(value);
            }
        });

        vec.into_iter()
            .for_each(|id| action(data.2.get(id as usize).unwrap()))
    }

    pub fn read_buf<B>(&self, mut buf: B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        let mut this = self.0.write();

        let i = buf.get_u8();
        let taken_data = this.data.take();
        let mut data = this.get_compatible_data(taken_data, i as usize);

        rimecraft_edcode::Update::update(&mut data.2, buf)?;

        this.data = Some(data);

        Ok(())
    }

    pub fn write_buf<B>(&self, mut buf: B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        let data_r = self.0.read();
        let data_r1 = data_r.data.as_ref();
        let data = data_r1.unwrap();

        buf.put_u8(data.1.element_bits() as u8);
        data.2.encode(&mut buf)?;
        data.1.data().encode(&mut buf)?;

        Ok(())
    }

    pub fn count<F>(&self, counter: F)
    where
        F: Fn(T, usize),
    {
        let this = self.0.read();
        let data = this.data.as_ref().unwrap();

        if data.2.len() == 1 {
            counter(data.2.get(0).unwrap(), data.2.len());
        } else {
            let mut map: Vec<(u64, usize)> = Vec::new();
            let ptr = &mut map as *mut Vec<(u64, usize)>;

            data.1.iter().for_each(|key| {
                let m = unsafe { ptr.as_mut().unwrap() };

                if let Some(entry) = m.iter_mut().find(|e| e.0 == key) {
                    entry.1 += 1;
                } else {
                    m.push((key, 1));
                }
            });

            map.into_iter().for_each(|(key, value)| {
                counter(data.2.get(key as usize).unwrap(), value);
            })
        }
    }
}

struct ContainerInner<'a, T: 'a>
where
    T: Eq + Copy + Hash,
{
    ids: rimecraft_primitives::Ref<'a, dyn rimecraft_collections::Index<T> + Send + Sync>,
    data: Option<Data<'a, T>>,
    provider: Provider,
}

impl<'a, T: 'a> ContainerInner<'a, T>
where
    T: Eq + Copy + Hash,
{
    fn get_compatible_data(&self, previous: Option<Data<'a, T>>, bits: usize) -> Data<'a, T> {
        let data_provider = self.provider.create_provider(self.ids.0, bits);

        if let Some(data) = previous {
            if data_provider == data.0 {
                return data;
            }
        }

        data_provider.create_data(self.ids.0, self.provider.container_size())
    }
}
