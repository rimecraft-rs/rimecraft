pub mod entity;
mod event;

use std::{hash::Hash, ops::Deref};

use crate::{
    prelude::*,
    registry::{Registration, RegistryAccess},
};

pub use event::*;

use once_cell::sync::Lazy;

//TODO: Build and freeze STATE_IDS

/// An `ID <-> BlockState` list.
pub static STATE_IDS: Lazy<crate::util::Freezer<crate::collections::IdList<SharedBlockState>>> =
    once_cell::sync::Lazy::new(|| crate::util::Freezer::new(crate::collections::IdList::new()));

/// Represents a block.
#[derive(Clone, Copy)]
pub struct Block {
    id: usize,
    pub states: crate::util::Ref<'static, crate::state::States<BlockState>>,
}

impl Block {
    pub fn new(states: Vec<(crate::state::property::Property, u8)>) -> anyhow::Result<Self> {
        Ok(Self {
            id: 0,
            states: {
                let mut builder = crate::state::StatesBuilder::new();
                let mut map = std::collections::HashMap::new();
                for state in states {
                    builder.add(state.0.clone())?;
                    map.insert(state.0, state.1);
                }
                builder.build((), map)
            }
            .into(),
        })
    }

    pub fn default_state(&self) -> SharedBlockState {
        crate::state::Shared {
            entries: self.states,
            value: crate::Ref(self.states.0.default_state()),
        }
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

impl RegistryAccess for Block {
    fn registry() -> &'static crate::registry::Registry<Self> {
        crate::registry::BLOCK.deref()
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
        let id = Id::deserialize(deserializer)?;
        Ok(crate::registry::BLOCK.get_from_id(&id).map_or_else(
            || {
                tracing::debug!("Tried to load invalid block: {id}");
                *crate::registry::BLOCK.default_entry().1.deref()
            },
            |e| e.1.deref().clone(),
        ))
    }
}

impl Default for Block {
    fn default() -> Self {
        crate::registry::BLOCK.default_entry().1.deref().clone()
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

/// An immutable state for a [`Block`].
pub struct BlockState {
    block: std::sync::atomic::AtomicUsize,
    state: crate::state::State,
    fluid_state: once_cell::sync::OnceCell<crate::state::Shared<crate::fluid::FluidState>>,
}

impl BlockState {
    /// Get block of this state.
    pub fn block(&self) -> Block {
        *crate::registry::BLOCK
            .get_from_raw(self.block.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap()
            .deref()
    }

    pub fn fluid_state(&self) -> crate::state::Shared<crate::fluid::FluidState> {
        *self.fluid_state.get_or_init(|| todo!())
    }
}

impl From<((), crate::state::State)> for BlockState {
    fn from((_, value): ((), crate::state::State)) -> Self {
        Self {
            block: std::sync::atomic::AtomicUsize::new(0),
            state: value,
            fluid_state: once_cell::sync::OnceCell::new(),
        }
    }
}

impl Deref for BlockState {
    type Target = crate::state::State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

/// A shared [`BlockState`] with states reference count and the index.
pub type SharedBlockState = crate::state::Shared<BlockState>;
