//! `rimecraft-arena` integrations.

#![cfg(feature = "arena")]

use arena::Arena;
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    pub struct TestArenaItem;
}

#[derive(Debug, Default)]
pub struct TestArena<V> {
    storage: SlotMap<TestArenaItem, V>,
}

impl<V> TestArena<V> {
    pub fn new() -> Self {
        Self {
            storage: SlotMap::with_key(),
        }
    }
}

impl<V> Arena for TestArena<V>
where
    V: Send + Sync,
{
    type Item = V;
    type Handle = TestArenaItem;

    fn insert(&mut self, item: Self::Item) -> Self::Handle {
        self.storage.insert(item)
    }

    fn get(&self, handle: Self::Handle) -> Option<&Self::Item> {
        self.storage.get(handle)
    }

    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item> {
        self.storage.get_mut(handle)
    }

    fn remove(&mut self, handle: Self::Handle) -> Option<Self::Item> {
        self.storage.remove(handle)
    }

    fn contains(&self, handle: Self::Handle) -> bool {
        self.storage.contains_key(handle)
    }

    fn len(&self) -> usize {
        self.storage.len()
    }

    fn clear(&mut self) {
        self.storage.clear();
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::Handle, &'a Self::Item)> + 'a> {
        Box::new(self.storage.iter())
    }

    fn iter_mut<'a>(
        &'a mut self,
    ) -> Box<dyn Iterator<Item = (Self::Handle, &'a mut Self::Item)> + 'a> {
        Box::new(self.storage.iter_mut())
    }
}
