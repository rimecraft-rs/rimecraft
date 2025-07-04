//! Features corresponding to vanilla Minecraft's `GameEvent`.
//!
//! This is completely different from `rimecraft-event` as the former one is way more generalized.

//TODO: funtion of dispatch manager, which requires interacting with a world instance.

use std::{fmt::Debug, hash::Hash};

use ahash::AHashSet;
use glam::DVec3;
use ident_hash::{HashTableExt as _, IHashSet};
use local_cx::dyn_codecs::{Any, EdcodeCodec, SerdeCodec, UnsafeEdcodeCodec, UnsafeSerdeCodec};
use maybe::Maybe;
use parking_lot::Mutex;
use rimecraft_block::BlockState;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

use crate::{Entity, World, chunk::ChunkCx};

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
        world: &World<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        emitter_pos: DVec3,
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
        world: &World<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        emitter_pos: DVec3,
    ) -> ListenResult;
    fn _erased_range(&self) -> u32;
    fn _erased_position_source(&self) -> &dyn PositionSource<'w, Cx>;
    fn _erased_trigger_order(&self) -> TriggerOrder;

    // Flatten position source
    fn _erased_ps_pos(&self, world: &World<'w, Cx>) -> Option<DVec3>;
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
        world: &World<'w, Cx>,
        event: GameEvent<'w, Cx>,
        emitter: Emitter<'_, 'w, Cx>,
        emitter_pos: DVec3,
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
    fn _erased_ps_pos(&self, world: &World<'w, Cx>) -> Option<DVec3> {
        self.position_source().pos(world)
    }
    fn _erased_ps_ty(&self) -> PositionSourceType<'w, Cx> {
        self.position_source().ty()
    }
}

/// Listening result of [`Listener::listen`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
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

type BoxedListener<'w, Cx> = Box<dyn ErasedListener<'w, Cx> + Send + Sync + 'w>;

/// Dispatcher of game events and their listeners.
///
/// This dispatcher comes with internal mutability and multi-thread support,
/// as like the one in vanilla Minecraft.
pub struct Dispatcher<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    listeners: Mutex<Vec<BoxedListener<'w, Cx>>>,
    buf: Mutex<DispatcherBuf<'w, Cx>>,
}

/// Key of a dispatched listener.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ListenerKey(*const ());

unsafe impl Send for ListenerKey {}
unsafe impl Sync for ListenerKey {}

impl Hash for ListenerKey {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.addr().hash(state);
    }
}

struct DispatcherBuf<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    push: Vec<BoxedListener<'w, Cx>>,
    pop: IHashSet<ListenerKey>,
}

impl<'w, Cx> Dispatcher<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Creates a new dispatcher.
    pub fn new() -> Self {
        Self {
            listeners: Mutex::new(Vec::new()),
            buf: Mutex::new(DispatcherBuf {
                push: Vec::new(),
                pop: IHashSet::new(),
            }),
        }
    }

    /// Whether this dispatcher is empty.
    pub fn is_empty(&self) -> bool {
        self.listeners.lock().is_empty()
    }

    /// Pushes a listener to this dispatcher.
    pub fn push<T>(&self, listener: T) -> ListenerKey
    where
        T: Listener<'w, Cx> + Send + Sync + 'w,
    {
        let boxed = Box::new(listener);
        let ptr = std::ptr::from_ref::<T>(&*boxed) as *const ();
        if let Some(mut guard) = self.listeners.try_lock() {
            guard.push(boxed);
        } else {
            self.buf.lock().push.push(boxed);
        }
        ListenerKey(ptr)
    }

    /// Removes a listener from this dispatcher if present.
    pub fn remove(&self, key: ListenerKey) {
        if let Some(mut guard) = self.listeners.try_lock() {
            // maybe we can switch to thing like hashmap to reduce complexity?
            if let Some(i) = guard
                .iter()
                .enumerate()
                .find(|(_, l)| std::ptr::from_ref(&***l) as *const () == key.0)
                .map(|(i, _)| i)
            {
                guard.swap_remove(i);
            }
        } else {
            self.buf.lock().pop.insert(key);
        }
    }

    /// Dispatches all the listeners in this dispatcher.
    /// Firing event to any listener should be done by given `callback`, who receives listener and its position.
    ///
    /// Returns whether the callback was triggered.
    pub fn dispatch<F>(&self, world: &World<'w, Cx>, pos: DVec3, mut callback: F) -> bool
    where
        F: for<'env> FnMut(&'env mut dyn ErasedListener<'w, Cx>, DVec3),
    {
        let mut vg = self.listeners.lock();
        let mut visited = false;
        for listener in &mut *vg {
            if let Some(sp) = listener._erased_ps_pos(world) {
                let d = sp.floor().distance_squared(pos.floor());
                let i = listener._erased_range().pow(2) as f64;
                if d <= i {
                    callback(&mut **listener, sp);
                    visited = true;
                }
            }
        }

        let mut buf = self.buf.lock();
        vg.extend(std::mem::take(&mut buf.push));
        vg.retain(|l| {
            !buf.pop
                .contains(&ListenerKey(std::ptr::from_ref(&**l) as *const ()))
        });
        buf.pop.clear();

        visited
    }
}

impl<'w, Cx> Default for Dispatcher<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'w, Cx> Debug for Dispatcher<'w, Cx>
where
    Cx: ChunkCx<'w, Id: Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher")
            .field(
                "listeners",
                &self
                    .listeners
                    .lock()
                    .iter()
                    .map(|l| l._erased_ps_ty())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

/// A property of a game event listener which provides position of an in-game object.
pub trait PositionSource<'w, Cx>: Any
where
    Cx: ChunkCx<'w>,
{
    /// Gets position of this source.
    fn pos(&self, world: &World<'w, Cx>) -> Option<DVec3>;

    /// Gets the type of this position source.
    fn ty(&self) -> PositionSourceType<'w, Cx>;
}

/// Raw type of a `PositionSource` with its type erased, consisting of codecs.
#[derive(Debug)]
pub struct RawPositionSourceType<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    serde: UnsafeSerdeCodec<
        dyn PositionSource<'w, Cx> + Send + Sync + 'w,
        dyn PositionSource<'w, Cx> + 'w,
    >,
    packet: UnsafeEdcodeCodec<
        dyn PositionSource<'w, Cx> + Send + Sync + 'w,
        dyn PositionSource<'w, Cx> + 'w,
    >,
}

/// Registry entry of [`RawPositionSourceType`].
pub type PositionSourceType<'a, Cx> =
    Reg<'a, <Cx as ProvideIdTy>::Id, RawPositionSourceType<'a, Cx>>;

impl<'a, Cx> RawPositionSourceType<'a, Cx>
where
    Cx: ChunkCx<'a>,
{
    /// Creates a new position source type from codecs.
    #[inline]
    pub const fn new<T>(
        serde_codec: SerdeCodec<
            T,
            dyn PositionSource<'a, Cx> + Send + Sync + 'a,
            dyn PositionSource<'a, Cx> + 'a,
        >,
        packet_codec: EdcodeCodec<
            T,
            dyn PositionSource<'a, Cx> + Send + Sync + 'a,
            dyn PositionSource<'a, Cx> + 'a,
        >,
    ) -> Self {
        Self {
            serde: serde_codec.codec,
            packet: packet_codec.codec,
        }
    }
}

#[allow(missing_docs)]
mod sealed {
    pub trait Sealed<Cx> {}
}

mod _edcode {
    use edcode2::{Buf, BufMut, Decode, Encode};
    use local_cx::{LocalContext, LocalContextExt, WithLocalCx, dyn_cx::AsDynamicContext};
    use rimecraft_registry::Registry;

    use crate::chunk::ChunkCx;

    use super::{PositionSource, PositionSourceType, RawPositionSourceType};

    impl<'w, Cx, B, L> Encode<WithLocalCx<B, L>> for dyn PositionSource<'w, Cx> + 'w
    where
        Cx: ChunkCx<'w>,
        L: AsDynamicContext,
        B: BufMut,
    {
        fn encode(&self, buf: WithLocalCx<B, L>) -> Result<(), edcode2::BoxedError<'static>> {
            let WithLocalCx {
                local_cx,
                mut inner,
            } = buf;
            let ty = self.ty();
            ty.encode(&mut inner)?;
            let dyn_cx = local_cx.as_dynamic_context();
            (ty.packet.encode)(self, &mut inner, unsafe { dyn_cx.as_unsafe_cx() })
        }
    }

    impl<'de, 'w, Cx, B, L> Decode<'de, WithLocalCx<B, L>>
        for Box<dyn PositionSource<'w, Cx> + Send + Sync + 'w>
    where
        Cx: ChunkCx<'w>,
        L: LocalContext<&'w Registry<Cx::Id, RawPositionSourceType<'w, Cx>>> + AsDynamicContext,
        B: Buf,
    {
        fn decode(buf: WithLocalCx<B, L>) -> Result<Self, edcode2::BoxedError<'de>> {
            let WithLocalCx {
                local_cx,
                mut inner,
            } = buf;
            let ty = PositionSourceType::decode(local_cx.with(&mut inner))?;
            let cx = local_cx.as_dynamic_context();
            (ty.packet.decode)(&mut inner, unsafe { cx.as_unsafe_cx() })
        }
    }
}

mod _serde {
    use std::marker::PhantomData;

    use local_cx::{
        LocalContext, LocalContextExt, WithLocalCx,
        dyn_cx::AsDynamicContext,
        serde::{DeserializeWithCx, SerializeWithCx, TYPE_KEY},
    };
    use rimecraft_registry::Registry;
    use serde::{Deserialize, Serialize, ser::SerializeMap};

    use crate::{chunk::ChunkCx, event::game_event::PositionSourceType};

    use super::{PositionSource, RawPositionSourceType};

    impl<'w, Cx, L> SerializeWithCx<L> for dyn PositionSource<'w, Cx> + 'w
    where
        Cx: ChunkCx<'w, Id: Serialize>,
        L: AsDynamicContext,
    {
        fn serialize_with_cx<S>(&self, serializer: WithLocalCx<S, L>) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let WithLocalCx { local_cx, inner } = serializer;
            let mut map = inner.serialize_map(None)?;
            let ty = self.ty();
            map.serialize_entry(TYPE_KEY, &ty)?;
            let cx = local_cx.as_dynamic_context();
            erased_serde::serialize(
                (ty.serde.ser)(&unsafe { cx.as_unsafe_cx() }.with(self)),
                serde::__private::ser::FlatMapSerializer(&mut map),
            )?;
            map.end()
        }
    }

    struct Visitor<'w, L, Cx>(L, PhantomData<&'w Cx>);

    impl<'de, 'w, L, Cx> serde::de::Visitor<'de> for Visitor<'w, L, Cx>
    where
        L: LocalContext<&'w Registry<Cx::Id, RawPositionSourceType<'w, Cx>>> + AsDynamicContext,
        Cx: ChunkCx<'w, Id: Deserialize<'de>>,
    {
        type Value = Box<dyn PositionSource<'w, Cx> + Send + Sync + 'w>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "position source with a type key dispatched")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            use serde::__private::de::Content;
            use serde::de::Error;
            let mut buf: Vec<(Content<'de>, Content<'de>)> =
                map.size_hint().map_or_else(Vec::new, Vec::with_capacity);
            let mut ty: Option<PositionSourceType<'w, Cx>> = None;
            while let Some(key) = map.next_key::<Content<'de>>()? {
                match &key {
                    Content::String(val) => {
                        if val == TYPE_KEY {
                            ty = Some(map.next_value_seed(self.0.with(PhantomData))?);
                            continue;
                        }
                    }
                    Content::Str(val) => {
                        if *val == TYPE_KEY {
                            ty = Some(map.next_value_seed(self.0.with(PhantomData))?);
                            continue;
                        }
                    }
                    _ => {}
                }
                buf.push((key, map.next_value()?))
            }
            let ty = ty.ok_or_else(|| A::Error::missing_field("type"))?;
            let cx = self.0.as_dynamic_context();
            (ty.serde.de)(
                &mut <dyn erased_serde::Deserializer<'de>>::erase(
                    serde::__private::de::ContentDeserializer::<'de, A::Error>::new(Content::Map(
                        buf,
                    )),
                ),
                unsafe { cx.as_unsafe_cx() },
            )
            .map_err(A::Error::custom)
        }
    }

    impl<'de, 'w, Cx, L> DeserializeWithCx<'de, L>
        for Box<dyn PositionSource<'w, Cx> + Send + Sync + 'w>
    where
        L: LocalContext<&'w Registry<Cx::Id, RawPositionSourceType<'w, Cx>>> + AsDynamicContext,
        Cx: ChunkCx<'w, Id: Deserialize<'de>>,
    {
        fn deserialize_with_cx<D>(deserializer: WithLocalCx<D, L>) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let WithLocalCx { local_cx, inner } = deserializer;
            inner.deserialize_map(Visitor(local_cx, PhantomData))
        }
    }
}
