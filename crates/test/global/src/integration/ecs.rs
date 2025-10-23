//! `rimecraft-ecs` integrations.

#![cfg(feature = "ecs")]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use ecs::World;

#[derive(Default)]
pub struct TestWorld {
    next_id: u32,
    components: HashMap<(u32, TypeId), Box<dyn Any + Send + Sync>>,
}

impl TestWorld {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            components: HashMap::new(),
        }
    }
}

impl World for TestWorld {
    type Entity = u32;

    fn spawn(&mut self) -> Self::Entity {
        let entity = self.next_id;
        self.next_id += 1;
        entity
    }

    fn despawn(&mut self, _entity: Self::Entity) {}

    fn has_entity(&self, entity: Self::Entity) -> bool {
        entity < self.next_id
    }

    fn all_entities(&self) -> Vec<Self::Entity> {
        (0..self.next_id).collect::<Vec<_>>()
    }

    fn add_component<C>(&mut self, entity: Self::Entity, component: C)
    where
        C: ecs::Component,
    {
        self.components
            .insert((entity, TypeId::of::<C>()), Box::new(component));
    }

    fn remove_component<C>(&mut self, entity: Self::Entity)
    where
        C: ecs::Component,
    {
        self.components.remove(&(entity, TypeId::of::<C>()));
    }

    fn component<C>(&self, entity: Self::Entity) -> Option<&C>
    where
        C: ecs::Component,
    {
        self.components
            .get(&(entity, TypeId::of::<C>()))?
            .downcast_ref()
    }

    fn component_mut<C>(&mut self, entity: Self::Entity) -> Option<&mut C>
    where
        C: ecs::Component,
    {
        self.components
            .get_mut(&(entity, TypeId::of::<C>()))?
            .downcast_mut()
    }
}
