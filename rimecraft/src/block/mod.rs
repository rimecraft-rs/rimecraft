mod event;

use std::{hash::Hash, ops::Deref};

use crate::{prelude::*, registry::Registration};

pub use event::*;

/// Represents a block.
#[derive(Clone)]
pub struct Block {
    id: usize,
    pub states: std::sync::Arc<crate::state::States<BlockState>>,
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
                builder.build((), map)
            }),
        })
    }
}

impl Registration for Block {
    fn accept(&mut self, id: usize) {
        self.id = id;
        self.states
            .states()
            .iter()
            .for_each(|state| state.block.store(id, std::sync::atomic::Ordering::Relaxed))
    }

    fn raw_id(&self) -> usize {
        self.id
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
        crate::registry::BLOCK
            .get_from_raw(self.raw_id())
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
                crate::registry::BLOCK.default().1.deref().clone()
            },
            |e| e.1.deref().clone(),
        ))
    }
}

impl Default for Block {
    fn default() -> Self {
        crate::registry::BLOCK.default().1.deref().clone()
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
    state: crate::state::State,
}

impl BlockState {
    /// Get block of this state.
    pub fn block(&self) -> Block {
        crate::registry::BLOCK
            .get_from_raw(self.block.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap()
            .deref()
            .clone()
    }
}

impl From<((), crate::state::State)> for BlockState {
    fn from((_, value): ((), crate::state::State)) -> Self {
        Self {
            block: std::sync::atomic::AtomicUsize::new(0),
            state: value,
        }
    }
}

impl Deref for BlockState {
    type Target = crate::state::State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
