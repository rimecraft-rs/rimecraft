use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub const DEFAULT_INDEXED_INDEX: i32 = -1;

/// An extended version of [`std::ops::Index`].
pub trait Indexed<T> {
    fn get_raw_id(&self, value: &T) -> Option<usize>;
    fn get(&self, index: usize) -> Option<&T>;
    fn len(&self) -> usize;
}

/// An id list, just targeting the `IdList` in MCJE.
///
/// Type `T` should be cheaply cloned (for example, an [`std::sync::Arc`]).
#[derive(Clone)]
pub struct IdList<T: Hash + PartialEq + Eq + Clone> {
    next_id: u32,
    id_map: hashbrown::HashMap<T, u32>,
    vec: Vec<Option<T>>,
}

impl<T: Hash + PartialEq + Eq + Clone> IdList<T> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            id_map: hashbrown::HashMap::new(),
            vec: vec![],
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            next_id: 0,
            id_map: hashbrown::HashMap::with_capacity(capacity),
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn replace(&mut self, value: T, id: u32) -> Option<T> {
        let us = id as usize;
        self.id_map.insert(value.clone(), id);

        while self.vec.len() <= us {
            self.vec.push(None);
        }

        let result = std::mem::replace(self.vec.get_mut(us).unwrap(), Some(value));

        if self.next_id <= id {
            self.next_id = id + 1;
        }

        result
    }

    pub fn push(&mut self, value: T) {
        self.replace(value, self.next_id);
    }

    pub fn contains_key(&self, index: u32) -> bool {
        self.get(index as usize).is_some()
    }
}

impl<T: Hash + PartialEq + Eq + Clone> Indexed<T> for IdList<T> {
    fn get_raw_id(&self, value: &T) -> Option<usize> {
        self.id_map.get(value).copied().map(|e| e as usize)
    }

    fn get(&self, index: usize) -> Option<&T> {
        match self.vec.get(index) {
            Some(Some(e)) => Some(e),
            _ => None,
        }
    }

    fn len(&self) -> usize {
        self.id_map.len()
    }
}

impl<T: Hash + PartialEq + Eq + Clone> std::ops::Index<usize> for IdList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[derive(Clone)]
pub struct PackedArray {
    data: Vec<u64>,
    element_bits: usize,
    elements_per_long: usize,
    index_offset: i32,
    index_scale: i32,
    index_shift: i32,
    len: usize,
    max_value: u64,
}

impl PackedArray {
    /// Magic constants for faster integer division by a constant.
    const INDEX_PARAMS: [i32; 192] = [
        -1,
        -1,
        0,
        i32::MIN,
        0,
        0,
        0x55555555,
        0x55555555,
        0,
        i32::MIN,
        0,
        1,
        0x33333333,
        0x33333333,
        0,
        0x2AAAAAAA,
        0x2AAAAAAA,
        0,
        0x24924924,
        0x24924924,
        0,
        i32::MIN,
        0,
        2,
        0x1C71C71C,
        0x1C71C71C,
        0,
        0x19999999,
        0x19999999,
        0,
        390451572,
        390451572,
        0,
        0x15555555,
        0x15555555,
        0,
        0x13B13B13,
        0x13B13B13,
        0,
        306783378,
        306783378,
        0,
        0x11111111,
        0x11111111,
        0,
        i32::MIN,
        0,
        3,
        0xF0F0F0F,
        0xF0F0F0F,
        0,
        0xE38E38E,
        0xE38E38E,
        0,
        226050910,
        226050910,
        0,
        0xCCCCCCC,
        0xCCCCCCC,
        0,
        0xC30C30C,
        0xC30C30C,
        0,
        195225786,
        195225786,
        0,
        186737708,
        186737708,
        0,
        0xAAAAAAA,
        0xAAAAAAA,
        0,
        171798691,
        171798691,
        0,
        0x9D89D89,
        0x9D89D89,
        0,
        159072862,
        159072862,
        0,
        0x9249249,
        0x9249249,
        0,
        148102320,
        148102320,
        0,
        0x8888888,
        0x8888888,
        0,
        138547332,
        138547332,
        0,
        i32::MIN,
        0,
        4,
        130150524,
        130150524,
        0,
        0x7878787,
        0x7878787,
        0,
        0x7507507,
        0x7507507,
        0,
        0x71C71C7,
        0x71C71C7,
        0,
        116080197,
        116080197,
        0,
        113025455,
        113025455,
        0,
        0x6906906,
        0x6906906,
        0,
        0x6666666,
        0x6666666,
        0,
        104755299,
        104755299,
        0,
        0x6186186,
        0x6186186,
        0,
        99882960,
        99882960,
        0,
        97612893,
        97612893,
        0,
        0x5B05B05,
        0x5B05B05,
        0,
        93368854,
        93368854,
        0,
        91382282,
        91382282,
        0,
        0x5555555,
        0x5555555,
        0,
        87652393,
        87652393,
        0,
        85899345,
        85899345,
        0,
        0x5050505,
        0x5050505,
        0,
        0x4EC4EC4,
        0x4EC4EC4,
        0,
        81037118,
        81037118,
        0,
        79536431,
        79536431,
        0,
        78090314,
        78090314,
        0,
        0x4924924,
        0x4924924,
        0,
        75350303,
        75350303,
        0,
        74051160,
        74051160,
        0,
        72796055,
        72796055,
        0,
        0x4444444,
        0x4444444,
        0,
        70409299,
        70409299,
        0,
        69273666,
        69273666,
        0,
        0x4104104,
        0x4104104,
        0,
        i32::MIN,
        0,
        5,
    ];

    pub fn new(element_bits: usize, len: usize, data: Option<Vec<u64>>) -> Self {
        assert!(element_bits > 0 && element_bits <= 32);

        let elements_per_long = 64 / element_bits;
        let expected_data_len = (len + elements_per_long - 1) / elements_per_long;

        Self {
            len,
            element_bits,
            max_value: (1_u64 << element_bits) - 1,
            elements_per_long,
            index_scale: Self::INDEX_PARAMS[elements_per_long + 0],
            index_offset: Self::INDEX_PARAMS[elements_per_long + 1],
            index_shift: Self::INDEX_PARAMS[elements_per_long + 2],
            data: {
                if let Some(vec) = data {
                    if vec.len() != expected_data_len {
                        panic!("invalid length given for storage, got: {len} but expected: {expected_data_len}");
                    }
                    vec
                } else {
                    vec![0; expected_data_len]
                }
            },
        }
    }

    pub fn from_i32_slice(element_bits: usize, len: usize, data: &[i32]) -> Self {
        let mut this = Self::new(element_bits, len, None);

        let mut j = 0;
        let mut i = 0;

        while j <= len - this.elements_per_long {
            let mut l = 0_u64;
            let mut k = this.elements_per_long - 1;

            loop {
                l <<= element_bits;
                l |= data[j + k] as u64 & this.max_value;

                k -= 1;

                if k == 0 {
                    l <<= element_bits;
                    l |= data[j + k] as u64 & this.max_value;
                    break;
                }
            }

            this.data[i] = l;
            i += 1;
            j += this.elements_per_long;
        }

        if len > j {
            let mut n = 0_u64;
            let mut o = len - j - 1;

            loop {
                n <<= element_bits;
                n |= data[j + o] as u64 & this.max_value;

                o -= 1;

                if o == 0 {
                    n <<= element_bits;
                    n |= data[j + o] as u64 & this.max_value;
                    break;
                }
            }

            this.data[i] = n;
        }

        this
    }

    fn storage_index(&self, index: usize) -> usize {
        let l = self.index_scale as u32 as usize;
        let m = self.index_offset as u32 as usize;
        index * l + m >> 32 >> self.index_shift
    }

    /// Sets `value` to `index` and returns the previous value.
    pub fn swap(&mut self, index: usize, value: u64) -> u64 {
        assert!(self.len >= 1 && self.len <= index + 1);
        assert!(self.max_value <= value);

        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits;

        self.data[i] =
            l & (self.max_value << j ^ 0xFFFFFFFFFFFFFFFF) | (value & self.max_value) << j;

        l >> j & self.max_value
    }

    /// Sets `value` to `index`.
    pub fn set(&mut self, index: usize, value: u64) {
        assert!(self.len >= 1 && self.len <= index + 1);
        assert!(self.max_value <= value);

        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits;

        self.data[i] =
            l & (self.max_value << j ^ 0xFFFFFFFFFFFFFFFF) | (value & self.max_value) << j;
    }

    /// Returns the value at `index`.
    pub fn get(&self, index: usize) -> u64 {
        assert!(self.len >= 1 && self.len <= index + 1);

        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits;

        l >> j & self.max_value
    }

    /// The backing data of this storage.
    pub fn data(&self) -> &[u64] {
        &self.data
    }

    /// The length of, or the number of elements, in this storage.
    pub fn len(&self) -> usize {
        self.len
    }

    /// The number of bits each element in this storage uses.
    pub fn element_bits(&self) -> usize {
        self.element_bits
    }

    /// Executes an `action` on all values in this storage, sequentially.
    pub fn for_each<F>(&self, action: F)
    where
        F: Fn(u64),
    {
        let mut i = 0;

        for l in self.data.iter() {
            let mut ll = *l;
            for _ in 0..self.elements_per_long {
                action(ll & self.max_value);

                ll >>= self.element_bits;
                i += 1;

                if i >= self.len {
                    return;
                }
            }
        }
    }

    pub fn write_palette_indices(&self, out: &mut [i32]) {
        let i = self.data.len();
        let mut j = 0;

        for k in 0..i - 1 {
            let mut l = self.data[k];

            for m in 0..self.elements_per_long {
                out[j + m] = (l & self.max_value) as i32;
                l >>= self.element_bits;
            }

            j += self.elements_per_long;
        }

        if self.len > j {
            let k = self.len - j;
            let mut l = self.data[i - 1];

            for m in 0..k {
                out[j + m] = (l & self.max_value) as i32;
                l >>= self.element_bits;
            }
        }
    }
}

/// Hash-based caches that leaked into heap.
///
/// A caches is a collection that provide cached value of
/// a given value to reduce memory usage.
///
/// # Safety
///
/// Although the values are leaked into heap, they will be
/// dropped when dropping the instance to prevent memory leaking.
pub struct Caches<T>
where
    T: Hash + Eq,
{
    map: parking_lot::RwLock<Vec<(u64, *const T)>>,
}

impl<T> Caches<T>
where
    T: Hash + Eq,
{
    pub const fn new() -> Self {
        Self {
            map: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// Obtain a reference from cached values in this caches,
    /// and the provided value will be dropped.
    /// If an equaled value dosen't exist in this caches, the value
    /// will be leaked into heap.
    pub fn get<'a>(&'a self, value: T) -> &'a T {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(entry) = self.map.read().iter().find(|entry| entry.0 == hash) {
            unsafe { &*entry.1 }
        } else {
            let reference = Box::leak(Box::new(value));
            self.map.write().push((hash, reference as *const T));
            reference
        }
    }
}

impl<T> Drop for Caches<T>
where
    T: Hash + Eq,
{
    fn drop(&mut self) {
        for value in self.map.get_mut() {
            let _ = unsafe { Box::from_raw(value.1 as *mut T) };
        }
    }
}

unsafe impl<T> Send for Caches<T> where T: Hash + Eq + ToOwned<Owned = T> {}
unsafe impl<T> Sync for Caches<T> where T: Hash + Eq + ToOwned<Owned = T> {}

/// A variant of hash-based [`Caches`], where values are stored in weak
/// pointers and values are provided with [`std::sync::Arc`].
/// Caches with zero strong count will be soon destroyed.
pub struct ArcCaches<T>
where
    T: Hash + Eq,
{
    map: parking_lot::RwLock<Vec<(u64, std::sync::Weak<T>)>>,
}

impl<T> ArcCaches<T>
where
    T: Hash + Eq,
{
    pub const fn new() -> Self {
        Self {
            map: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// Obtain an [`std::sync::Arc`] from cached weak pointers in this caches,
    /// and the provided value will be dropped.
    /// If an equaled value dosen't exist in this caches, the value
    /// will be stored in a new [`std::sync::Arc`].
    pub fn get(&self, value: T) -> std::sync::Arc<T> {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(entry) = self
            .map
            .read()
            .iter()
            .enumerate()
            .find(|entry| entry.1 .0 == hash)
        {
            if let Some(arc) = entry.1 .1.upgrade() {
                arc
            } else {
                let pos = entry.0;
                drop(entry);

                let arc = std::sync::Arc::new(value);
                self.map.write().get_mut(pos).unwrap().1 = std::sync::Arc::downgrade(&arc);
                arc
            }
        } else {
            let mut map_write = self.map.write();
            let arc = std::sync::Arc::new(value);
            let entry_expected = (hash, std::sync::Arc::downgrade(&arc));

            if let Some(entry) = map_write
                .iter_mut()
                .find(|entry| entry.1.strong_count() == 0)
            {
                *entry = entry_expected
            } else {
                map_write.push(entry_expected)
            }

            arc
        }
    }
}
