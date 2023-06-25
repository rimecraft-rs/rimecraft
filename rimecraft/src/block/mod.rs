mod event;

use crate::prelude::*;

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

impl serde::Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::registry::ITEM
            .get_from_raw(self.id())
            .unwrap()
            .key()
            .value()
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = Identifier::deserialize(deserializer)?;
        Ok(crate::registry::BLOCK.get_from_id(&id).map_or_else(
            || {
                tracing::debug!("Tried to load invalid item: {id}");
                Block(crate::registry::BLOCK.default().1 .0)
            },
            |e| Self(e.0),
        ))
    }
}

impl Default for Block {
    fn default() -> Self {
        Self(crate::registry::BLOCK.default().0)
    }
}

pub struct BlockState {
    block: Block,
}

impl BlockState {
    /// Get block of this state.
    pub fn block(&self) -> Block {
        self.block
    }
}
