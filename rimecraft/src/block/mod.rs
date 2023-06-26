mod event;

use std::{hash::Hash, ops::Deref};

use crate::prelude::*;

pub use event::*;

/// Represents a block.
#[derive(Clone)]
pub struct Block {
    id: usize,
    states: std::sync::Arc<crate::state::States<BlockState>>,
}

impl Block {
    pub fn new(states: Vec<(crate::state::property::Property, u8)>) -> anyhow::Result<Self> {
        Ok(Self {
            id: 0,
            states: std::sync::Arc::new({
                let mut builder = crate::state::StatesBuilder::new();
                let mut map = std::collections::HashMap::new();
                for state in states {
                    builder.add(state.0.clone())?;
                    map.insert(state.0, state.1);
                }
                builder.build(0, map)
            }),
        })
    }

    /// Raw id of this block.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl crate::registry::Registration for Block {
    fn accept(&mut self, id: usize) {
        self.id = id;
        self.states
            .states()
            .iter()
            .for_each(|state| state.block.store(id, std::sync::atomic::Ordering::Relaxed))
    }
}

impl crate::item::AsItem for Block {
    fn as_item(&self) -> crate::item::Item {
        todo!()
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
                tracing::debug!("Tried to load invalid block: {id}");
                crate::registry::BLOCK.default().1.deref().deref().clone()
            },
            |e| e.1.deref().deref().clone(),
        ))
    }
}

impl Default for Block {
    fn default() -> Self {
        crate::registry::BLOCK.default().1.deref().deref().clone()
    }
}

impl Eq for Block {}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub struct BlockState {
    block: std::sync::atomic::AtomicUsize,
    state: crate::state::RawState,
}

impl BlockState {
    /// Get block of this state.
    pub fn block(&self) -> Block {
        crate::registry::BLOCK
            .get_from_raw(self.block.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap()
            .deref()
            .deref()
            .clone()
    }
}

impl From<(usize, crate::state::RawState)> for BlockState {
    fn from(value: (usize, crate::state::RawState)) -> Self {
        Self {
            block: std::sync::atomic::AtomicUsize::new(value.0),
            state: value.1,
        }
    }
}

impl Deref for BlockState {
    type Target = crate::state::RawState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
