//! Features corresponding to vanilla Minecraft's `GameEvent`.
//!
//! This is completely different from `rimecraft-event` as the former one is way more generalized.

use std::fmt::Debug;

use maybe::Maybe;
use rimecraft_block::BlockState;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

use crate::{Entity, ServerWorld, chunk::ChunkCx};

/// Raw type of a [`GameEvent`], consisting of its notification radius.
#[derive(Debug)]
pub struct RawGameEvent {
    notification_radius: u32,
}

impl RawGameEvent {
    /// Creates a new raw game event instance.
    #[inline]
    pub const fn new(notification_radius: u32) -> Self {
        Self {
            notification_radius,
        }
    }

    /// Gets the underlying notification radius.
    #[inline]
    pub const fn notification_radius(&self) -> u32 {
        self.notification_radius
    }
}

impl Default for RawGameEvent {
    #[inline]
    fn default() -> Self {
        Self {
            notification_radius: 16,
        }
    }
}

/// A game event in the form of registry entry.
pub type GameEvent<'w, Cx> = Reg<'w, <Cx as ProvideIdTy>::Id, RawGameEvent>;

/// A game event listener listens to [`GameEvent`]s from dispatchers.
pub trait Listener<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Listens to an incoming game event.
    fn listen(
        &mut self,
        world: &ServerWorld<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        //TODO vec3d emitter position
    ) -> ListenResult;

    /// Gets the range of this listener, in blocks.
    fn range(&self) -> u32;

    /// Gets this listener's trigger order.
    #[inline(always)]
    fn trigger_order(&self) -> TriggerOrder {
        Default::default()
    }
}

/// Listening result of [`Listener::listen`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenResult {
    /// Listener has accepted the event.
    Accepted,
    /// Listener has not accepted the event.
    Unaccepted,
}

impl ListenResult {
    /// Whether the listener has accepted the event.
    #[inline(always)]
    pub const fn is_accepted(self) -> bool {
        matches!(self, Self::Accepted)
    }
}

/// Trigger Order of a [`Listener`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum TriggerOrder {
    /// Unspecified. The default order.
    #[default]
    Unspecified,
    /// Trigger by distance.
    Distance,
}

/// Emitter of a game event.
pub struct Emitter<'event, 'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    src: Option<Maybe<'event, Entity<'w, Cx>>>,
    dst: Option<BlockState<'w, Cx>>,
}

impl<'event, 'w, Cx> Emitter<'event, 'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Creates a new emitter from source entity and affected state.
    #[inline]
    pub fn new(src: Option<&'event Entity<'w, Cx>>, dst: Option<BlockState<'w, Cx>>) -> Self {
        Self {
            src: src.map(Maybe::Borrowed),
            dst,
        }
    }

    /// Creates a new emitter from owned source entity and affected state.
    #[inline]
    pub fn from_owned(src: Option<Entity<'w, Cx>>, dst: Option<BlockState<'w, Cx>>) -> Self {
        Self {
            src: src.map(|e| Maybe::Owned(maybe::SimpleOwned(e))),
            dst,
        }
    }

    /// Gets the source entity of this game event emitter.
    #[inline]
    pub fn source_entity(&self) -> Option<&Entity<'w, Cx>> {
        self.src.as_deref()
    }

    /// Gets the affected state of this game event emitter.
    #[inline]
    pub fn affected_state(&self) -> Option<BlockState<'w, Cx>> {
        self.dst
    }

    /// Drops the `'event` lifetime by cloning the underlying entity if borrowed.
    pub fn drop_lifetime<'any>(self) -> Emitter<'any, 'w, Cx> {
        Emitter {
            dst: self.dst,
            src: self.src.map(|m| Maybe::<'any, _, _>::Owned(m.into_owned())),
        }
    }

    /// Obtains a borrowed emitter from this event emitter.
    #[inline]
    pub fn to_ref(&self) -> Emitter<'_, 'w, Cx> {
        Emitter {
            src: self.src.as_ref().map(|m| Maybe::Borrowed(&**m)),
            dst: self.dst,
        }
    }
}

impl<'w, Cx> Debug for Emitter<'_, 'w, Cx>
where
    Cx: ChunkCx<'w, Id: Debug, BlockStateExt: Debug> + Debug,
    Entity<'w, Cx>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Emitter")
            .field(&self.src)
            .field(&self.dst)
            .finish()
    }
}
