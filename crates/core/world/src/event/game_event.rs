//! Features corresponding to vanilla Minecraft's `GameEvent`.
//!
//! This is completely different from `rimecraft-event` as the former one is way more generalized.

use std::fmt::Debug;

use glam::Vec3;
use local_cx::dyn_codecs::{EdcodeCodec, SerdeCodec, UnsafeEdcodeCodec, UnsafeSerdeCodec};
use maybe::Maybe;
use rimecraft_block::BlockState;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

use crate::{Entity, ServerWorld, World, chunk::ChunkCx};

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
///
/// See [`ErasedListener`] for type-erasure.
pub trait Listener<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Type of this listener's [`PositionSource`].
    type PositionSource: PositionSource<'w, Cx>;

    /// Listens to an incoming game event.
    fn listen(
        &mut self,
        world: &ServerWorld<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        emitter_pos: Vec3,
    ) -> ListenResult;

    /// Gets the range of this listener, in blocks.
    fn range(&self) -> u32;

    /// Gets this listener's position source.
    fn position_source(&self) -> &Self::PositionSource;

    /// Gets this listener's trigger order.
    #[inline(always)]
    fn trigger_order(&self) -> TriggerOrder {
        Default::default()
    }
}

/// Dyn-compatible [`Listener`].
#[allow(missing_docs)]
pub trait ErasedListener<'w, Cx>: sealed::Sealed<Cx>
where
    Cx: ChunkCx<'w>,
{
    fn _erased_listen(
        &mut self,
        world: &ServerWorld<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        emitter_pos: Vec3,
    ) -> ListenResult;
    fn _erased_range(&self) -> u32;
    fn _erased_position_source(&self) -> &dyn PositionSource<'w, Cx>;
    fn _erased_trigger_order(&self) -> TriggerOrder;

    // Flatten position source
    fn _erased_ps_pos(&self, world: &World<'w, Cx>) -> Option<Vec3>;
    fn _erased_ps_ty(&self) -> PositionSourceType<'w, Cx>;
}

impl<'w, T, Cx> sealed::Sealed<Cx> for T
where
    T: Listener<'w, Cx>,
    Cx: ChunkCx<'w>,
{
}

impl<'w, T, Cx> ErasedListener<'w, Cx> for T
where
    T: Listener<'w, Cx>,
    Cx: ChunkCx<'w>,
{
    fn _erased_listen(
        &mut self,
        world: &ServerWorld<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        emitter_pos: Vec3,
    ) -> ListenResult {
        self.listen(world, event, emitter, emitter_pos)
    }

    fn _erased_range(&self) -> u32 {
        self.range()
    }

    fn _erased_position_source(&self) -> &dyn PositionSource<'w, Cx> {
        self.position_source()
    }

    fn _erased_trigger_order(&self) -> TriggerOrder {
        self.trigger_order()
    }

    fn _erased_ps_pos(&self, world: &World<'w, Cx>) -> Option<Vec3> {
        self.position_source().pos(world)
    }

    fn _erased_ps_ty(&self) -> PositionSourceType<'w, Cx> {
        self.position_source().ty()
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

/// A property of a game event listener which provides position of an in-game object.
pub trait PositionSource<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Gets position of this source.
    fn pos(&self, world: &World<'w, Cx>) -> Option<Vec3>;

    /// Gets the type of this position source.
    fn ty(&self) -> PositionSourceType<'w, Cx>;
}

/// Raw type of a `PositionSource` with its type erased, consisting of codecs.
#[derive(Debug)]
pub struct RawPositionSourceType<'a> {
    serde: UnsafeSerdeCodec<'a>,
    packet: UnsafeEdcodeCodec<'a>,
}

/// Registry entry of [`RawPositionSourceType`].
pub type PositionSourceType<'a, Cx> = Reg<'a, <Cx as ProvideIdTy>::Id, RawPositionSourceType<'a>>;

impl<'a> RawPositionSourceType<'a> {
    /// Creates a new position source type from codecs.
    #[inline]
    pub const fn new<T>(serde_codec: SerdeCodec<'a, T>, packet_codec: EdcodeCodec<'a, T>) -> Self {
        Self {
            serde: serde_codec.to_unsafe(),
            packet: packet_codec.to_unsafe(),
        }
    }
}

#[allow(missing_docs)]
mod sealed {
    pub trait Sealed<Cx> {}
}
