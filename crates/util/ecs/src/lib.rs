//! Entity Component System for Rimecraft.

use std::fmt::Debug;

use rimecraft_global_cx::GlobalContext;

pub trait ProvideEcsTy: GlobalContext {
    /// The world type.
    type World: World;
}

pub trait Entity: Copy + Eq + Debug {}

impl<T> Entity for T where T: Copy + Eq + Debug {}

pub trait Component: 'static + Send + Sync {}

pub trait World {
    type Entity: Entity;

    fn spawn(&mut self) -> Self::Entity;

    fn despawn(&mut self, entity: Self::Entity);

    fn has_entity(&self, entity: Self::Entity) -> bool;

    fn all_entities(&self) -> Vec<Self::Entity>;

    fn add_component<C>(&mut self, entity: Self::Entity, component: C)
    where
        C: Component;

    fn remove_component<C>(&mut self, entity: Self::Entity)
    where
        C: Component;

    fn component<C>(&self, entity: Self::Entity) -> Option<&C>
    where
        C: Component;

    fn component_mut<C>(&mut self, entity: Self::Entity) -> Option<&mut C>
    where
        C: Component;

    fn query<Q>(&mut self) -> Q::Iter<'_>
    where
        Q: WorldQuery<Self>,
        Self: Sized,
    {
        Q::iter(self)
    }
}

pub trait WorldQuery<W: World> {
    type Item<'a>
    where
        W: 'a;

    type Iter<'a>: Iterator<Item = Self::Item<'a>>
    where
        W: 'a;

    fn iter(world: &mut W) -> Self::Iter<'_>;
}
