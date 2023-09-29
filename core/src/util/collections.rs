use crate::net::Encode;
use std::{borrow::Borrow, hash::Hash, ops::Deref};
use tracing::instrument;

pub const DEFAULT_INDEXED_INDEX: i32 = -1;

/// An extended version of [`std::ops::Index`].
pub trait Indexed<T> {
    fn raw_id(&self, value: &T) -> Option<usize>;
    fn get(&self, index: usize) -> Option<&T>;
    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// An id list, just targeting the `IdList` in MCJE.
///
/// Type `T` should be cheaply cloned (for example, an [`std::sync::Arc`]).
#[derive(Clone)]
pub struct IdList<T: Hash + PartialEq + Eq + Clone> {
    next_id: u32,
    id_map: std::collections::HashMap<T, u32>,
    vec: Vec<Option<T>>,
}

impl<T: Hash + PartialEq + Eq + Clone> IdList<T> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            next_id: 0,
            id_map: std::collections::HashMap::with_capacity(capacity),
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

impl<T> Default for IdList<T>
where
    T: Hash + Eq + Clone,
{
    #[inline]
    fn default() -> Self {
        Self {
            next_id: 0,
            id_map: std::collections::HashMap::new(),
            vec: vec![],
        }
    }
}

impl<T: Hash + PartialEq + Eq + Clone> Indexed<T> for IdList<T> {
    fn raw_id(&self, value: &T) -> Option<usize> {
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

        l.wrapping_shr(j as u32) & self.max_value
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

        assert!(packed_array.iter().any(|e| e == 2));
        assert!(packed_array.iter().any(|e| e == 7));
    }
}

/// Thread safe and hash-based caches.
///
/// A caches is a collection that provide cached value of
/// a given value to reduce memory usage.
pub struct Caches<T>
where
    T: Hash + Eq,
{
    map: dashmap::DashSet<Box<T>>,
}

impl<T> Caches<T>
where
    T: Hash + Eq,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Obtain a reference from cached values in this caches,
    /// and the provided value will be dropped.
    /// If an equaled value doesn't exist in this caches, the value
    /// will be leaked into heap.
    pub fn get(&self, value: T) -> &T {
        if let Some(v) = self.map.get(&value) {
            unsafe { &*(v.deref().deref() as *const T) }
        } else {
            let boxed = Box::new(value);
            let ptr = boxed.deref() as *const T;
            self.map.insert(boxed);
            unsafe { &*ptr }
        }
    }

    #[inline]
    pub fn contains<Q>(&self, value: &T) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized,
    {
        self.map.contains(value)
    }
}

impl<T> Default for Caches<T>
where
    T: Hash + Eq,
{
    fn default() -> Self {
        Self {
            map: dashmap::DashSet::new(),
        }
    }
}

/// A variant of hash-based [`Caches`], where values are stored in weak
/// pointers and values are provided with [`std::sync::Arc`].
///
/// Caches with zero strong count will be soon destroyed.
pub struct ArcCaches<T>
where
    T: Hash + Eq + 'static,
{
    map: dashmap::DashSet<arc_caches_imp::WeakNode<'static, T>>,
}

impl<T> ArcCaches<T>
where
    T: Hash + Eq + 'static,
{
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Obtain an [`std::sync::Arc`] from cached weak pointers in this caches,
    /// and the provided value will be dropped.
    /// If an equaled value dosen't exist in this caches, the value
    /// will be stored in a new [`std::sync::Arc`].
    pub fn get(&self, value: T) -> std::sync::Arc<T> {
        if let Some(v) = self.map.get(&arc_caches_imp::WeakNode::Ref(unsafe {
            &*(&value as *const T)
        })) {
            if let arc_caches_imp::WeakNode::Stored(weak) = v.deref() {
                weak.upgrade().expect("invalid weak pointer")
            } else {
                unreachable!()
            }
        } else {
            let arc = std::sync::Arc::new(value);
            self.map
                .insert(arc_caches_imp::WeakNode::Stored(std::sync::Arc::downgrade(
                    &arc,
                )));

            arc
        }
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        self.map.contains(&arc_caches_imp::WeakNode::Ref(unsafe {
            &*(value as *const T)
        }))
    }
}

impl<T> Default for ArcCaches<T>
    where
        T: Hash + Eq,
{
    fn default() -> Self {
        Self {
            map: dashmap::DashSet::new(),
        }
    }
}

mod arc_caches_imp {
    use std::ops::Deref;
    use std::{hash::Hash, sync::Weak};

    pub enum WeakNode<'a, T> {
        Stored(Weak<T>),
        Ref(&'a T),
    }

    impl<T> Hash for WeakNode<'_, T>
    where
        T: Hash,
    {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            match self {
                WeakNode::Stored(value) => {
                    if let Some(v) = value.upgrade() {
                        v.hash(state)
                    }
                }
                WeakNode::Ref(value) => value.hash(state),
            }
        }
    }

    impl<T> PartialEq for WeakNode<'_, T>
    where
        T: Hash + Eq,
    {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (WeakNode::Stored(value0), WeakNode::Stored(value1)) => value0.ptr_eq(value1),
                (WeakNode::Ref(value0), WeakNode::Stored(value1)) => {
                    value1.upgrade().map_or(false, |e| e.deref() == *value0)
                }
                (WeakNode::Stored(value0), WeakNode::Ref(value1)) => {
                    value0.upgrade().map_or(false, |e| e.deref() == *value1)
                }
                (WeakNode::Ref(value0), WeakNode::Ref(value1)) => value0 == value1,
            }
        }
    }

    impl<T> Eq for WeakNode<'_, T> where T: Hash + Eq {}
}

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
}

pub trait Weighted {
    fn weight(&self) -> u32;
}
