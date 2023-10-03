mod magic_nums;

#[cfg(test)]
mod tests;

/// A packed array of integers.
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
    /// Creates a new packed array.
    ///
    /// # Panics
    ///
    /// - Panics if `element_bits` is greater than 64.
    /// - Panics if the length of data got is invalid.
    pub fn new(element_bits: usize, len: usize, data: Option<Vec<u64>>) -> Self {
        assert!(
            element_bits <= 64,
            "element bits should not greater than 64"
        );

        let elements_per_long = 64 / element_bits;
        let expected_data_len = (len + elements_per_long - 1) / elements_per_long;

        use magic_nums::INDEX_PARAMS;

        Self {
            len,
            element_bits,
            max_value: (1_u64 << element_bits) - 1,
            elements_per_long,

            index_scale: INDEX_PARAMS[elements_per_long],
            index_offset: INDEX_PARAMS[elements_per_long + 1],
            index_shift: INDEX_PARAMS[elements_per_long + 2],

            data: {
                if let Some(vec) = data {
                    assert_eq!(vec.len(), expected_data_len, "invalid length given for storage, got: {len} but expected: {expected_data_len}");
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
        (index * l + m) >> 32 >> self.index_shift
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
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Indicates whether this storage is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of bits each element in this storage uses.
    #[inline]
    pub fn element_bits(&self) -> usize {
        self.element_bits
    }

    /// Gets the iterator of this storage.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        let mut iter = self.data.iter();

        Iter {
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

/// Iterator over [`PackedArray`].
pub struct Iter<'a> {
    instance: &'a PackedArray,
    i: usize,
    iter: std::slice::Iter<'a, u64>,
    epl_index: usize,
    l: u64,
}

impl Iterator for Iter<'_> {
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
