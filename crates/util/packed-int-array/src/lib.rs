//! `PackedIntegerArray` in Rust.

mod consts;
mod iter;

pub use iter::{IntoIter, Iter};

use crate::consts::INDEX_PARAMS;

/// A packed container for storing small integers.
#[derive(Debug, Clone)]
pub struct PackedIntArray {
    data: Vec<u64>,
    element_bits: usize,
    max: u64,
    len: usize,
    elements_per_long: usize,

    index_scale: isize,
    index_offset: isize,
    index_shift: isize,
}

impl PackedIntArray {
    /// Creates a new `PackedIntArray` with given `element_bits`, `len` and `raw`
    /// packed data.
    ///
    /// # Panics
    ///
    /// - Panics if the given `element_bits` is not in range `(0, 32]`.
    /// - Panics if length of the given raw data slice is not equal to
    ///   `(len + 64 / element_bits - 1) / (64 / element_bits)`.
    pub fn from_packed(element_bits: usize, len: usize, raw: Option<&[u64]>) -> Self {
        assert!(
            0 < element_bits && element_bits <= 32,
            "element bits should in range (0, 32]"
        );

        let max = (1u64 << element_bits) - 1;
        let elements_per_long = 64 / element_bits;
        let i = 3 * (elements_per_long - 1);
        let index_scale = INDEX_PARAMS[i] as isize;
        let index_offset = INDEX_PARAMS[i + 1] as isize;
        let index_shift = INDEX_PARAMS[i + 2] as isize;
        let j = (len + elements_per_long - 1) / elements_per_long;

        if let Some(data) = raw {
            assert_eq!(data.len(), j, "invalid length given for storage");
        }

        Self {
            data: raw.map(Vec::from).unwrap_or_else(|| vec![0; j]),
            element_bits,
            max,
            len,
            elements_per_long,
            index_scale,
            index_offset,
            index_shift,
        }
    }

    #[inline]
    const fn storage_index(&self, index: usize) -> usize {
        let l = self.index_scale as u32 as usize;
        let m = self.index_offset as u32 as usize;
        (index * l + m) >> 32 >> self.index_shift
    }

    /// Sets the data at given `index` with given value and returns the old one.
    ///
    /// # Panics
    ///
    /// Panics if the given value is greater than the internal max value.
    pub fn set(&mut self, index: usize, value: u32) -> Option<u32> {
        assert!(
            value as u64 <= self.max,
            "given value {} could not be greater than max value {}",
            value,
            self.max
        );

        if index >= self.len {
            return None;
        }

        let i = self.storage_index(index);
        let l = &mut self.data[i];
        let lo = *l;
        let j = (index - i * self.elements_per_long) * self.element_bits;
        *l = *l & !(self.max << j) | (value as u64 & self.max) << j;
        Some((lo >> j & self.max) as u32)
    }

    /// Gets the value at target index.
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len {
            return None;
        }
        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits;
        Some((l >> j & self.max) as u32)
    }

    /// Gets the inner packed data of this array.
    #[inline]
    pub fn data(&self) -> &[u64] {
        &self.data
    }

    /// Gets the inner packed mutable data of this array.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u64] {
        &mut self.data
    }

    /// Gets the length of this array.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether this array is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets an iterator over this array.
    pub fn iter(&self) -> Iter<'_> {
        if self.is_empty() {
            Iter {
                array: self,
                iter: self.data.iter(),
                inner: iter::IterInner {
                    l: 0,
                    j: self.elements_per_long,
                    times: 0,
                },
            }
        } else {
            Iter {
                array: self,
                iter: self.data[1..].iter(),
                inner: iter::IterInner {
                    l: self.data[0],
                    j: 0,
                    times: 0,
                },
            }
        }
    }
}

impl IntoIterator for PackedIntArray {
    type Item = u32;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        if self.is_empty() {
            IntoIter {
                iter: self.data.into_iter(),
                inner: iter::IterInner {
                    l: 0,
                    j: self.elements_per_long,
                    times: 0,
                },
                element_bits: self.element_bits,
                elements_per_long: self.elements_per_long,
                max: self.max,
                len: self.len,
            }
        } else {
            let mut iter = self.data.into_iter();
            let first = iter.next().unwrap();
            IntoIter {
                iter,
                inner: iter::IterInner {
                    l: first,
                    j: 0,
                    times: 0,
                },
                element_bits: self.element_bits,
                elements_per_long: self.elements_per_long,
                max: self.max,
                len: self.len,
            }
        }
    }
}

impl<'a> IntoIterator for &'a PackedIntArray {
    type Item = u32;

    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests;
