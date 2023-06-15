mod event;

pub use event::*;

/// Represents a block.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block(usize);

impl Block {
    pub fn new() -> Self {
        Self(0)
    }

    /// Raw id of this block.
    pub fn id(&self) -> usize {
        self.0
    }
}

impl crate::registry::Registration for Block {
    fn accept(&mut self, id: usize) {
        self.0 = id
    }
}

impl crate::item::AsItem for Block {
    fn as_item(&self) -> crate::item::Item {
        EVENTS.read().block_item_map(*self).item()
    }
}
