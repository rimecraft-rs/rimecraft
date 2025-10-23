//! Entity Component System for Rimecraft.

use std::fmt::Debug;
use std::hash::Hash;

use rimecraft_global_cx::GlobalContext;

/// Provides the concrete ECS types for a global context.
pub trait ProvideEcsTy: GlobalContext {
    /// The world type used by this context.
    type World: World;
}

/// An entity identifier. Typically a thin, copyable value (integer or newtype).
///
/// It must be Copy, Eq and Hash so it can be stored in maps/sets. Debug is
/// required to aid debugging and tests.
pub trait Entity: Copy + Eq + Hash + Debug {}

impl<T> Entity for T where T: Copy + Eq + Hash + Debug {}

/// The world trait. Runtimes provide concrete implementations.
///
/// Designed to be minimal: spawns, despawns, basic component CRUD and
/// query helpers. Implementations may choose any internal storage.
pub trait World {
    /// The concrete entity identifier type for this world.
    type Entity: Entity;

    /// Create a new entity and return its id.
    fn spawn(&mut self) -> Self::Entity;

    /// Remove an entity and all its components.
    fn despawn(&mut self, entity: Self::Entity);

    /// Whether the entity currently exists in the world.
    fn has_entity(&self, entity: Self::Entity) -> bool;

    /// Return a list of all entities. Implementations may return in arbitrary order.
    fn all_entities(&self) -> Vec<Self::Entity>;

    /// Insert a component for the given entity.
    fn insert<C>(&mut self, entity: Self::Entity, component: C)
    where
        C: 'static + Send + Sync;

    /// Remove a component of type C from the given entity.
    fn remove<C>(&mut self, entity: Self::Entity)
    where
        C: 'static + Send + Sync;

    /// Get a shared reference to a component of type C for the entity.
    fn get<C>(&self, entity: Self::Entity) -> Option<&C>
    where
        C: 'static + Send + Sync;

    /// Get an exclusive reference to a component of type C for the entity.
    /// Mutable access may have different representation per-world. Implementors
    /// can choose an appropriate mutable wrapper (for example bevy's
    /// `Mut<'a, C>`) by setting the associated type `Mut<'a, C>` below.
    type Mut<'a, C>
    where
        Self: 'a,
        C: 'static + Send + Sync;

    fn get_mut<C>(&mut self, entity: Self::Entity) -> Option<Self::Mut<'_, C>>
    where
        C: 'static + Send + Sync;

    /// Immutable query helper. By default it forwards to Q::iter.
    fn query<Q>(&self) -> Q::Iter<'_>
    where
        Q: WorldQuery<Self>,
        Self: Sized,
    {
        Q::iter(self)
    }

    /// Mutable query helper. Forwarding to Q::iter_mut when implemented.
    fn query_mut<Q>(&mut self) -> Q::IterMut<'_>
    where
        Q: WorldQueryMut<Self>,
        Self: Sized,
    {
        Q::iter_mut(self)
    }
}

/// A query over a world that yields items borrowed from the world.
///
/// Implementations should provide an iterator type which yields items with
/// the correct borrow semantics. For many simple queries, the iterator only
/// needs an immutable borrow; some queries require mutable access and should
/// implement `WorldQueryMut`.
pub trait WorldQuery<W: World> {
    /// Item yielded by the immutable query.
    type Item<'a>
    where
        W: 'a;

    /// Iterator type for immutable queries.
    type Iter<'a>: Iterator<Item = Self::Item<'a>>
    where
        W: 'a;

    /// Produce an iterator which borrows the world immutably.
    fn iter(world: &W) -> Self::Iter<'_>;
}

/// A mutable query over a world. Separate trait keeps the immutable API
/// ergonomic while allowing mutable-only queries.
pub trait WorldQueryMut<W: World> {
    /// Item yielded by the mutable query.
    type Item<'a>
    where
        W: 'a;

    /// Iterator type for mutable queries.
    type IterMut<'a>: Iterator<Item = Self::Item<'a>>
    where
        W: 'a;

    /// Produce an iterator which borrows the world mutably.
    fn iter_mut(world: &mut W) -> Self::IterMut<'_>;
}
