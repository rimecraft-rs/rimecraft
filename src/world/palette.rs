use crate::network::{Decode, Encode};

/// A palette maps objects from and to small integer IDs that uses less
/// number of bits to make storage smaller.
///
/// While the objects palettes handle are already represented by integer
/// IDs, shrinking IDs in cases where only a few appear can further reduce
/// storage space and network traffic volume.
pub struct Palette<'a, T: PartialEq + Eq + Copy + 'a> {
    ids: crate::Ref<'a, dyn crate::collections::Indexed<T> + Send + Sync>,
    inner: Inner<'a, T>,
}

impl<'a, T: 'a + PartialEq + Eq + Copy> Palette<'a, T> {
    /// Returns the ID of an object in this palette.
    pub fn index(&self, value: T) -> Option<usize> {
        match &self.inner {
            Inner::Vector(vec) => vec
                .iter()
                .position(|option| option.map_or(false, |e| e == value)),
            Inner::Indexed(ids) => ids.get_raw_id(&value),
            Inner::Singular(option) => option
                .map(|e| if e == value { Some(0) } else { None })
                .flatten(),
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
                vec.push(Some(value));
                vec.len() - 1
            }
            Inner::Indexed(_) => 0,
            Inner::Singular(option) => {
                *option = Some(value);
                0
            }
        })
    }

    /// Returns `true` if any entry in this palette passes the predicate.
    pub fn any<P>(&self, predicate: P) -> bool
    where
        P: Fn(T) -> bool,
    {
        match &self.inner {
            Inner::Vector(vec) => vec.iter().any(|e| e.map_or(false, |ee| predicate(ee))),
            Inner::Indexed(_) => true,
            Inner::Singular(option) => predicate(option.expect("use of an uninitialized palette")),
        }
    }

    /// Returns the object associated with the given `index`.
    pub fn get(&self, index: usize) -> Option<T> {
        match &self.inner {
            Inner::Vector(vec) => vec.get(index).map(|e| e.map(|ee| ee)).flatten(),
            Inner::Indexed(ids) => ids.0.get(index).copied(),
            Inner::Singular(option) => {
                if let (Some(e), 0) = (option, index) {
                    Some(*e)
                } else {
                    panic!("missing palette entry for id {index}")
                }
            }
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
                *vec = Vec::new();

                for _ in 0..crate::VarInt::decode(buf)? {
                    vec.push(
                        self.ids
                            .0
                            .get(crate::VarInt::decode(buf)? as usize)
                            .copied(),
                    );
                }
            }
            Inner::Indexed(_) => (),
            Inner::Singular(option) => {
                *option = Some(
                    self.ids
                        .get(crate::VarInt::decode(buf)? as usize)
                        .copied()
                        .ok_or_else(|| {
                            anyhow::anyhow!("raw id not found in the target id list.")
                        })?,
                );
            }
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
                                .get_raw_id(r)
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
            Inner::Singular(option) => {
                if let Some(entry) = option {
                    crate::VarInt(
                        self.ids
                            .get_raw_id(&entry)
                            .map(|e| e as i32)
                            .unwrap_or(crate::collections::DEFAULT_INDEXED_INDEX),
                    )
                    .encode(buf)?;
                } else {
                    panic!("use of an uninitialized palette");
                }
            }
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
                                .get_raw_id(r)
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
            Inner::Singular(option) => {
                if let Some(entry) = option {
                    crate::VarInt(
                        self.ids
                            .get_raw_id(&entry)
                            .map(|e| e as i32)
                            .unwrap_or(crate::collections::DEFAULT_INDEXED_INDEX),
                    )
                    .len()
                } else {
                    panic!("use of an uninitialized palette");
                }
            }
        }
    }

    /// Size of this palette.
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::Vector(vec) => vec.len(),
            Inner::Indexed(ids) => ids.len(),
            Inner::Singular(_) => 1,
        }
    }
}

impl<'a, T: 'a + PartialEq + Eq + Copy> Encode for Palette<'a, T> {
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.write_buf(buf)
    }
}

impl<'a, T: 'a + PartialEq + Eq + Copy> Clone for Palette<'a, T> {
    fn clone(&self) -> Self {
        Self {
            ids: self.ids,
            inner: self.inner.clone(),
        }
    }
}

enum Inner<'a, T: 'a + Copy> {
    Indexed(crate::Ref<'a, dyn crate::collections::Indexed<T> + Send + Sync>),
    Vector(Vec<Option<T>>),
    Singular(Option<T>),
}

impl<'a, T: 'a + Copy> Clone for Inner<'a, T> {
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
        entries: Vec<A>,
    ) -> Palette<'a, A>
    where
        A: 'a + PartialEq + Eq + Copy,
    {
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
                        vec[value.0] = Some(value.1);
                    }

                    vec
                }),
            },
            Variant::Singular => Palette {
                ids: crate::Ref(ids),
                inner: Inner::Singular(entries.get(0).copied()),
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

struct Data<'a, T: 'a>(DataProvider, Storage, Palette<'a, T>)
where
    T: PartialEq + Eq + Copy;

#[derive(Clone, Copy, PartialEq, Eq)]
struct DataProvider(Variant, usize);

impl DataProvider {
    fn create_data<'a, T>(
        self,
        ids: &'a (dyn crate::collections::Indexed<T> + Send + Sync),
        len: usize,
    ) -> Data<'a, T>
    where
        T: 'a + PartialEq + Eq + Copy,
    {
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
        ids: &(dyn crate::collections::Indexed<A>),
        bits: usize,
    ) -> DataProvider {
        match self {
            Provider::BlockState => match bits {
                0 => DataProvider(Variant::Singular, bits),
                1 | 2 | 3 | 4 => DataProvider(Variant::Vector, bits),
                5 | 6 | 7 | 8 => unimplemented!("bimap"),
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
        1 << self.edge_bits() * 3
    }
}

/// A paletted container stores objects in 3D voxels as small integer indices,
/// governed by "palettes" that map between these objects and indices.
pub struct Container<'a, T>(parking_lot::RwLock<ContainerInner<'a, T>>)
where
    T: 'a + PartialEq + Eq + Copy;

impl<'a, T: 'a + PartialEq + Eq + Copy> Container<'a, T> {
    pub fn new(
        ids: &'a (dyn crate::collections::Indexed<T> + Send + Sync),
        provider: Provider,
        variant: Variant,
        bits: usize,
        storage: Storage,
        entries: Vec<T>,
    ) -> Self {
        Self(parking_lot::RwLock::new(ContainerInner {
            ids: crate::Ref(ids),
            data: Some(Data(
                DataProvider(variant, bits),
                storage,
                variant.create(bits, ids, entries),
            )),
            provider,
        }))
    }

    pub fn from_initialize(
        ids: &'a (dyn crate::collections::Indexed<T> + Send + Sync),
        object: T,
        provider: Provider,
    ) -> Self {
        let mut this = ContainerInner {
            ids: crate::Ref(ids),
            data: None,
            provider,
        };

        this.data = Some(this.get_compatible_data(None, 0));
        this.data.as_mut().unwrap().2.index_or_insert(object);

        Self(parking_lot::RwLock::new(this))
    }

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

        data.1.for_each(|value| {
            let v = unsafe { ptr.as_mut().unwrap() };

            if !v.contains(&value) {
                v.push(value);
            }
        });

        vec.into_iter()
            .for_each(|id| action(data.2.get(id as usize).unwrap()))
    }

    pub fn read_buf<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        let mut this = self.0.write();

        let i = buf.get_u8();
        let taken_data = this.data.take();
        let mut data = this.get_compatible_data(taken_data, i as usize);

        data.2.read_buf(buf)?;

        this.data = Some(data);

        Ok(())
    }

    pub fn write_buf<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        let data_r = self.0.read();
        let data_r1 = data_r.data.as_ref();
        let data = data_r1.unwrap();

        buf.put_u8(data.1.element_bits() as u8);
        data.2.write_buf(buf)?;
        data.1.data().encode(buf)?;

        Ok(())
    }

    pub fn count(&self, counter: &mut impl ContainerCounter<T>) {
        let this = self.0.read();
        let data = this.data.as_ref().unwrap();

        if data.2.len() == 1 {
            counter.accept(data.2.get(0).unwrap(), data.2.len());
        } else {
            let mut map: Vec<(u64, usize)> = Vec::new();
            let ptr = &mut map as *mut Vec<(u64, usize)>;

            data.1.for_each(|key| {
                let m = unsafe { ptr.as_mut().unwrap() };

                if let Some(entry) = m.iter_mut().find(|e| e.0 == key) {
                    entry.1 += 1;
                } else {
                    m.push((key, 1));
                }
            });

            map.into_iter().for_each(|(key, value)| {
                counter.accept(data.2.get(key as usize).unwrap(), value);
            })
        }
    }
}

struct ContainerInner<'a, T: 'a + PartialEq + Eq + Copy> {
    ids: crate::Ref<'a, dyn crate::collections::Indexed<T> + Send + Sync>,
    data: Option<Data<'a, T>>,
    provider: Provider,
}

impl<'a, T: 'a + PartialEq + Eq + Copy> ContainerInner<'a, T> {
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

pub trait ContainerCounter<T> {
    fn accept(&mut self, value: T, count: usize);
}
