use std::{hash::Hash, ops::Deref};

use crate::{
    prelude::*,
    registry::{Registration, RegistryAccess},
};

/// Represents a type of fluid.
#[derive(Clone)]
pub struct Fluid {
    id: usize,
    states: std::sync::Arc<crate::state::States<FluidState>>,
}

impl Fluid {
    pub fn new(states: Vec<(crate::state::property::Property, u8)>) -> anyhow::Result<Self> {
        Ok(Self {
            id: 0,
            states: std::sync::Arc::new({
                let mut builder = crate::state::StatesBuilder::new();
                let mut map = hashbrown::HashMap::new();
                for state in states {
                    builder.add(state.0.clone())?;
                    map.insert(state.0, state.1);
                }
                builder.build((), map)
            }),
        })
    }

    /// Raw id of this fluid.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Registration for Fluid {
    fn accept(&mut self, id: usize) {
        self.id = id;
        self.states
            .states()
            .iter()
            .for_each(|state| state.fluid.store(id, std::sync::atomic::Ordering::Relaxed))
    }

    fn raw_id(&self) -> usize {
        self.id
    }
}

impl RegistryAccess for Fluid {
    fn registry() -> &'static crate::registry::Registry<Self> {
        crate::registry::FLUID.deref()
    }
}

impl serde::Serialize for Fluid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::registry::FLUID
            .get_from_raw(self.id())
            .unwrap()
            .key()
            .value()
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Fluid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = Identifier::deserialize(deserializer)?;
        match crate::registry::FLUID.get_from_id(&id) {
            Some(e) => Ok(e.1.deref().clone()),
            None => Ok(crate::registry::FLUID.default_entry().1.deref().clone()),
        }
    }
}

impl Eq for Fluid {}

impl PartialEq for Fluid {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Fluid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Default for Fluid {
    fn default() -> Self {
        crate::registry::FLUID.default_entry().1.deref().clone()
    }
}

pub struct FluidState {
    fluid: std::sync::atomic::AtomicUsize,
    state: crate::state::State,
}

impl FluidState {
    /// Get block of this state.
    pub fn fluid(&self) -> Fluid {
        crate::registry::FLUID
            .get_from_raw(self.fluid.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap()
            .deref()
            .clone()
    }
}

impl From<((), crate::state::State)> for FluidState {
    fn from((_, value): ((), crate::state::State)) -> Self {
        Self {
            fluid: std::sync::atomic::AtomicUsize::new(0),
            state: value,
        }
    }
}

impl Deref for FluidState {
    type Target = crate::state::State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
