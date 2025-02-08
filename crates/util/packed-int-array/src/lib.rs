//! `PackedIntegerArray` in Rust.

mod consts;
mod iter;

pub use iter::{IntoIter, Iter};

use crate::consts::INDEX_PARAMS;

/// A packed container for storing small integers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(alias = "PackedIntegerArray")]
#[doc(alias = "BitStorage")]
pub struct PackedIntArray {
    data: Vec<u64>,
    element_bits: u32,
    max: u64,
    len: usize,
    elements_per_long: usize,

    index_scale: isize,
    index_offset: isize,
    index_shift: isize,
}

impl PackedIntArray {
    /// Creates a new `PackedIntArray` with given `element_bits`, `len` and indices.
    ///
    /// # Panics
    ///
    /// See [`Self::from_packed`].
    ///
    /// # Errors
    ///
    /// See [`Self::from_packed`].
    pub fn new(element_bits: u32, len: usize, data: &[u32]) -> Result<Self, Error> {
        let mut this = Self::from_packed(element_bits, len, None)?;

        let mut i = 0;
        let mut jj = 0;
        for j in (0..len).step_by(this.elements_per_long) {
            let mut l = 0;

            for ii in data[j..j + this.elements_per_long].iter().copied().rev() {
                l <<= this.element_bits;
                l |= ii as u64 & this.max;
            }

            i += 1;
            this.data[i] = l;
            jj = j;
        }

        if len > jj {
            let m = len - jj;
            let mut n = 0;
            for o in data[jj..jj + m].iter().copied().rev() {
                n <<= this.element_bits;
                n |= o as u64 & this.max;
            }
            this.data[i] = n;
        }

        Ok(this)
    }

    /// Creates a new `PackedIntArray` with given `element_bits`, `len` and `raw`
    /// packed data.
    ///
    /// # Panics
    ///
    /// Panics if the given `element_bits` is not in range `(0, 32]`.
    ///
    /// # Errors
    ///
    /// Returns an error if length of the given raw data slice is not equal to
    /// `(len + 64 / element_bits - 1) / (64 / element_bits)`.
    pub fn from_packed(element_bits: u32, len: usize, raw: Option<&[u64]>) -> Result<Self, Error> {
        assert!(
            0 < element_bits && element_bits <= 32,
            "element bits should in range (0, 32]"
        );

        let max = (1u64 << element_bits) - 1;
        let elements_per_long = 64 / element_bits as usize;
        let i = 3 * (elements_per_long - 1);
        let index_scale = INDEX_PARAMS[i] as isize;
        let index_offset = INDEX_PARAMS[i + 1] as isize;
        let index_shift = INDEX_PARAMS[i + 2] as isize;
        let j = len.div_ceil(elements_per_long);

        if raw.is_some_and(|d| d.len() != j) {
            return Err(Error::InvalidLength {
                expected: j,
                actual: raw.map_or(0, <[u64]>::len),
            });
        }

        Ok(Self {
            data: raw.map(Vec::from).unwrap_or_else(|| vec![0; j]),
            element_bits,
            max,
            len,
            elements_per_long,
            index_scale,
            index_offset,
            index_shift,
        })
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
    pub fn swap(&mut self, index: usize, value: u32) -> Option<u32> {
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
        let j = (index - i * self.elements_per_long) * self.element_bits as usize;
        *l = *l & !(self.max << j) | (value as u64 & self.max) << j;
        Some((lo >> j & self.max) as u32)
    }

    /// Sets the data at given `index` with given value.
    ///
    /// # Panics
    ///
    /// Panics if the given value is greater than the internal max value.
    pub fn set(&mut self, index: usize, value: u32) {
        assert!(
            value as u64 <= self.max,
            "given value {} could not be greater than max value {}",
            value,
            self.max
        );

        if index >= self.len {
            return;
        }

        let i = self.storage_index(index);
        let j = (index - i * self.elements_per_long) * self.element_bits as usize;
        self.data[i] &= !(self.max << j) | (value as u64 & self.max) << j;
    }

    /// Gets the value at target index.
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len {
            return None;
        }
        let i = self.storage_index(index);
        let l = self.data[i];
        let j = (index - i * self.elements_per_long) * self.element_bits as usize;
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

    /// Gets `elements_per_long` value of this array.
    #[inline]
    pub fn elements_per_long(&self) -> usize {
        self.elements_per_long
    }

    /// Gets `max` value of this array.
    #[inline]
    pub fn max(&self) -> u64 {
        self.max
    }

    /// Gets `element_bits` value of this array.
    #[inline]
    pub fn element_bits(&self) -> u32 {
        self.element_bits
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

/// Error type for `PackedIntArray`.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Invalid length given for storage.
    InvalidLength {
        /// Expected length.
        expected: usize,
        /// Actual length.
        actual: usize,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "invalid length given for storage, expected: {}, actual: {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests;
