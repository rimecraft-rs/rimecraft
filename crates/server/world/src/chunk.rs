//! Server-only chunk stuff.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use block_entity::{BlockEntity, BlockEntityCell};
use global_cx::{Hold, HoldMut};
use ident_hash::IHashMap;
use local_cx::{LocalContext, dsyn_instanceof};
use parking_lot::{Mutex, MutexGuard};
use voxel_math::coord_section_from_block;
use world::chunk::{Chunk, ChunkCx, WorldChunk, WorldChunkAccess};

use crate::{behave::*, game_event};

/// Server-only chunk extension nested type.
pub struct NestedWorldChunkExt<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    game_event_dispatchers: Mutex<IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>>,
}

impl<'w, Cx> Debug for NestedWorldChunkExt<'w, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NestedExt")
            .field("game_event_dispatchers", &self.game_event_dispatchers)
            .finish()
    }
}

/// Server-only [`Chunk`] extension trait.
pub trait ServerChunk<'w, Cx>: Chunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Peeks the [`game_event::Dispatcher`] of given Y section coordinate.
    #[inline]
    fn peek_game_event_dispatcher<F, T>(&mut self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        let _ = y_section_coord;
        drop(f);
        None
    }

    /// Gets the [`game_event::Dispatcher`] of given Y section coordinate.
    #[inline]
    fn game_event_dispatcher(
        &mut self,
        y_section_coord: i32,
    ) -> Option<Arc<game_event::Dispatcher<'w, Cx>>> {
        self.peek_game_event_dispatcher(y_section_coord, Arc::clone)
    }
}

#[allow(missing_docs)]
pub trait ServerWorldChunkAccess<'w, Cx>: WorldChunkAccess<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    type GameEventDispatchersRead: Deref<
        Target = IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>,
    >;
    type GameEventDispatchersWrite: DerefMut<
        Target = IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>,
    >;

    fn read_game_event_dispatchers(self) -> Self::GameEventDispatchersRead;
    fn write_game_event_dispatchers(self) -> Self::GameEventDispatchersWrite;

    fn reclaim_server(&mut self) -> impl ServerWorldChunkAccess<'w, Cx>;
}

impl<'a, 'w, Cx> ServerWorldChunkAccess<'w, Cx> for &'a WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::WorldChunkExt: Hold<NestedWorldChunkExt<'w, Cx>>,
    WorldChunk<'w, Cx>: WorldChunkAccess<'w, Cx>,
{
    type GameEventDispatchersRead = Self::GameEventDispatchersWrite;

    type GameEventDispatchersWrite =
        MutexGuard<'a, IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>>;

    #[inline]
    fn read_game_event_dispatchers(self) -> Self::GameEventDispatchersRead {
        self.write_game_event_dispatchers()
    }

    #[inline]
    fn write_game_event_dispatchers(self) -> Self::GameEventDispatchersWrite {
        self.ext().get_held().game_event_dispatchers.lock()
    }

    #[inline]
    fn reclaim_server(&mut self) -> impl ServerWorldChunkAccess<'w, Cx> {
        *self
    }
}

impl<'a, 'w, Cx> ServerWorldChunkAccess<'w, Cx> for &'a mut WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::WorldChunkExt: HoldMut<NestedWorldChunkExt<'w, Cx>>,
    WorldChunk<'w, Cx>: WorldChunkAccess<'w, Cx>,
{
    type GameEventDispatchersRead = Self::GameEventDispatchersWrite;

    type GameEventDispatchersWrite = &'a mut IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>;

    #[inline]
    fn read_game_event_dispatchers(self) -> Self::GameEventDispatchersRead {
        self.write_game_event_dispatchers()
    }

    #[inline]
    fn write_game_event_dispatchers(self) -> Self::GameEventDispatchersWrite {
        self.ext_mut()
            .get_held_mut()
            .game_event_dispatchers
            .get_mut()
    }

    #[inline]
    fn reclaim_server(&mut self) -> impl ServerWorldChunkAccess<'w, Cx> {
        &mut **self
    }
}

pub(crate) fn wc_peek_game_event_dispatcher<'w, Cx, F, U>(
    this: impl ServerWorldChunkAccess<'w, Cx>,
    y_section_coord: i32,
    f: F,
) -> U
where
    F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> U,
    Cx: ChunkCx<'w>,
{
    let mut g = this.write_game_event_dispatchers();
    if let Some(d) = g.get(&y_section_coord) {
        f(d)
    } else {
        let d = Arc::new(game_event::Dispatcher::new());
        let result = f(&d);
        g.insert(y_section_coord, d);
        result
    }
}

pub(crate) fn wc_update_game_event_listener<'w, Cx>(
    mut this: impl ServerWorldChunkAccess<'w, Cx>,
    be_cell: &BlockEntityCell<'w, Cx>,
    be: impl Deref<Target = BlockEntity<'w, Cx>>,
) where
    Cx: ChunkCx<'w>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockEntityGetGameEventListener<Cx>>>,
{
    let y = be.pos().y();
    let listener_fn =
        dsyn_instanceof!(this.local_cx(), &*be => export BlockEntityGetGameEventListener<Cx>)
            .unwrap_or(default_block_entity_get_game_event_listener());

    // release the guard
    drop(be);

    if let Some(listener) = listener_fn(
        be_cell,
        this.local_cx(),
        BlockEntityGetGameEventListenerMarker,
    ) {
        wc_peek_game_event_dispatcher(this.reclaim_server(), coord_section_from_block(y), |d| {
            d.push_erased(match listener {
                maybe::Maybe::Borrowed(a) => a.clone(),
                maybe::Maybe::Owned(maybe::SimpleOwned(a)) => a,
            })
        });
    }
}

pub(crate) fn wc_remove_game_event_listener<'w, Cx>(
    mut this: impl ServerWorldChunkAccess<'w, Cx>,
    be_cell: &BlockEntityCell<'w, Cx>,
    be: impl Deref<Target = BlockEntity<'w, Cx>>,
) where
    Cx: ChunkCx<'w>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockEntityGetGameEventListener<Cx>>>,
{
    let y = be.pos().y();
    let listener_fn =
        dsyn_instanceof!(this.local_cx(), &*be => export BlockEntityGetGameEventListener<Cx>)
            .unwrap_or(default_block_entity_get_game_event_listener());

    // release the guard
    drop(be);

    if let Some(listener) = listener_fn(
        be_cell,
        this.local_cx(),
        BlockEntityGetGameEventListenerMarker,
    ) {
        wc_peek_game_event_dispatcher(this.reclaim_server(), coord_section_from_block(y), |d| {
            d.remove(game_event::ListenerKey::from_arc(&*listener));
        });
    }
}
