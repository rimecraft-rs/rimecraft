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
    pub fn new(element_bits: usize, len: usize, data: Option<Vec<u64>>) -> Self {
        use super::magic_num::INDEX_PARAMS;

        assert!(element_bits > 0 && element_bits <= 64);

        let elements_per_long = 64 / element_bits;
        let expected_data_len = (len + elements_per_long - 1) / elements_per_long;

        Self {
            len,
            element_bits,
            max_value: (1_u64 << element_bits) - 1,
            elements_per_long,
            index_scale: INDEX_PARAMS[elements_per_long + 0],
            index_offset: INDEX_PARAMS[elements_per_long + 1],
            index_shift: INDEX_PARAMS[elements_per_long + 2],
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
        assert!(self.len >= 1 && index < self.len);
        assert!(value <= self.max_value);

        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits;

        self.data[i] = l & !(self.max_value.wrapping_shl(j as u32))
            | (value & self.max_value).wrapping_shl(j as u32);

        l.wrapping_shr(j as u32) & self.max_value
    }

    /// Sets `value` to `index`.
    #[inline]
    pub fn set(&mut self, index: usize, value: u64) {
        self.swap(index, value);
    }

    /// Returns the value at `index`.
    pub fn get(&self, index: usize) -> u64 {
        assert!(self.len >= 1 && index < self.len);

        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits;

        l >> j & self.max_value
    }

    /// The backing data of this storage.
    #[inline]
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

    pub fn iter(&self) -> PackedArrayIter<'_> {
        let mut iter = self.data.iter();
        PackedArrayIter {
            instance: self,
            i: 0,
            epl_index: 0,
            l: iter.next().copied().unwrap_or_default(),
            iter,
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

pub struct PackedArrayIter<'a> {
    instance: &'a PackedArray,
    i: usize,
    iter: std::slice::Iter<'a, u64>,
    epl_index: usize,
    l: u64,
}

impl Iterator for PackedArrayIter<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i > self.instance.len {
            return None;
        }

        if self.epl_index >= self.instance.elements_per_long {
            if let Some(e) = self.iter.next() {
                self.l = *e;
                self.epl_index = 0;
            } else {
                return None;
            }
        } else {
            self.epl_index += 1;
        }

        let result = self.l & self.instance.max_value;

        self.l >>= self.instance.element_bits;
        self.i += 1;

        Some(result)
    }
}

#[cfg(test)]
mod packed_array_tests {
    use super::PackedArray;

    #[test]
    fn swap() {
        let mut packed_array = PackedArray::new(32, 64, None);
        packed_array.iter().for_each(|num| assert_eq!(num, 0));

        assert_eq!(packed_array.swap(4, 1), 0);
        assert_eq!(packed_array.swap(4, 2), 1);

        assert_eq!(packed_array.swap(35, 16), 0);
        assert_eq!(packed_array.swap(35, 7), 16);

        assert_eq!(packed_array.get(4), 2);
        assert_eq!(packed_array.get(35), 7);
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

        let read = self.map.read();

        if let Some(entry) = read.iter().find(|entry| entry.0 == hash) {
            unsafe { &*entry.1 }
        } else {
            drop(read);

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

unsafe impl<T> Send for Caches<T> where T: Hash + Eq + Send {}
unsafe impl<T> Sync for Caches<T> where T: Hash + Eq + Sync {}

#[cfg(test)]
mod tests_caches {
    use std::ops::Deref;

    use super::{ArcCaches, Caches};

    #[test]
    fn storing() {
        let caches: Caches<String> = Caches::new();
        let first_ptr = caches.get("1".to_string());

        assert_eq!(first_ptr, "1");
        assert_eq!(
            caches.get("1".to_string()) as *const String as usize,
            first_ptr as *const String as usize
        );
    }

    #[test]
    fn arc_storing() {
        let caches: ArcCaches<String> = ArcCaches::new();
        let first_ptr = caches.get("1".to_string());

        assert_eq!(first_ptr.deref(), "1");
        assert_eq!(
            caches.get("1".to_string()).deref() as *const String as usize,
            first_ptr.deref() as *const String as usize
        );
    }

    #[test]
    fn arc_destroying() {
        let caches: ArcCaches<String> = ArcCaches::new();
        let first_ptr = caches.get("1".to_string());
        let _second_ptr = caches.get("2".to_string());

        assert_eq!(caches.map.read().len(), 2);

        drop(first_ptr);
        let _third_ptr = caches.get("3".to_string());

        assert_eq!(caches.map.read().len(), 2);
    }
}

/// A variant of hash-based [`Caches`], where values are stored in weak
/// pointers and values are provided with [`std::sync::Arc`].
///
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

        let read = self.map.read();

        if let Some(entry) = read.iter().enumerate().find(|entry| entry.1 .0 == hash) {
            if let Some(arc) = entry.1 .1.upgrade() {
                arc
            } else {
                let pos = entry.0;
                drop(read);

                let arc = std::sync::Arc::new(value);
                self.map.write().get_mut(pos).unwrap().1 = std::sync::Arc::downgrade(&arc);
                arc
            }
        } else {
            drop(read);
            let mut write = self.map.write();
            let arc = std::sync::Arc::new(value);
            let entry_expected = (hash, std::sync::Arc::downgrade(&arc));

            if let Some(entry) = write.iter_mut().find(|entry| entry.1.strong_count() == 0) {
                *entry = entry_expected
            } else {
                write.push(entry_expected)
            }

            arc
        }
    }
}
