//! Arena to allocate items safely for Rimecraft.

use rimecraft_global_cx::GlobalContext;

pub trait ProvideArenaTy: GlobalContext {
    /// The concrete arena implementation associated with this context.
    type Arena: Arena;
}

pub trait Arena {
    /// The stored item type.
    type Item;

    /// The handle type used to look up items in the arena.
    type Handle: Copy + Eq + Send + Sync;

    /// Inserts an item into the arena and returns a handle that refers to it.
    fn insert(&mut self, item: Self::Item) -> Self::Handle;

    /// Tries to insert an item and returns an error value if insertion fails.
    ///
    /// # Errors
    ///
    /// Implementations which have capacity limits or other failure modes
    /// should return `Err(item)` to indicate the insertion failed and give
    /// the ownership of the item back to the caller.
    fn try_insert(&mut self, item: Self::Item) -> Result<Self::Handle, Self::Item> {
        Ok(self.insert(item))
    }

    /// Gets a shared reference to the item by handle, or `None` if the handle
    /// is invalid.
    fn get(&self, handle: Self::Handle) -> Option<&Self::Item>;

    /// Gets a mutable reference to the item by handle, or `None` if invalid.
    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item>;

    /// Removes the item associated with `handle` and return it if present.
    fn remove(&mut self, handle: Self::Handle) -> Option<Self::Item>;

    /// Whether the arena currently contains this handle.
    fn contains(&self, handle: Self::Handle) -> bool;

    /// Number of items stored in the arena.
    fn len(&self) -> usize;

    /// Whether the arena is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all items from the arena.
    fn clear(&mut self);

    /// Iterates over stored items and their handles. The iterator yields
    /// (handle, &item). Implementations should return an iterator that
    /// yields borrowed references to avoid requiring `Item: Copy`.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::Handle, &'a Self::Item)> + 'a>;

    /// Iterates mutably over stored items and their handles. Useful for bulk
    /// mutation without needing to repeatedly look up handles.
    fn iter_mut<'a>(
        &'a mut self,
    ) -> Box<dyn Iterator<Item = (Self::Handle, &'a mut Self::Item)> + 'a>;
}
