#[cfg(feature = "id_list")]
pub mod id_list;
#[cfg(feature = "packed_array")]
pub mod packed_array;

#[cfg(feature = "id_list")]
pub use id_list::IdList;
#[cfg(feature = "packed_array")]
pub use packed_array::PackedArray;

/// An extended version of [`std::ops::Index`].
pub trait Index<Idx>
where
    Idx: ?Sized,
{
    /// Gets the raw id from item.
    fn index_of(&self, value: &Idx) -> Option<usize>;

    /// Gets an item.
    fn get(&self, index: usize) -> Option<&Idx>;

    /// Gets the length of items contained by this container.
    fn len(&self) -> usize;

    /// Indicates whether this container is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
