//! Minimal, implementation-agnostic primitives for a command-driven UI.

use std::any::Any;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// Trait that indicates a key contains a generation token (generational id).
///
/// Implementors should provide a stable way to read the generation so callers
/// can detect stale/invalid handles (e.g. from a slotmap or generational index).
pub trait GenerationalKey {
    type Id: Copy + Eq;

    /// Return the generation / version token of this id.
    fn generation(&self) -> Self::Id;
}

/// A single UI mutation described as a value object.
pub trait Command<K>: Send {
    /// Applies this command to `store`.
    fn apply(&self, store: &mut dyn UiStore<K>);

    fn as_any(&self) -> &dyn Any;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

// Note: we intentionally do not provide a blanket impl for `Command` to
// avoid relying on unstable specialization. Concrete command types should
// implement `Command<Id>` explicitly.

/// Abstract queue for submitting commands.
pub trait CommandQueue<K>: Send + Sync {
    /// Submit a command.
    fn submit(&self, cmd: Box<dyn Command<K>>);

    /// Drain all queued commands into `out`.
    fn drain_into(&self, out: &mut Vec<Box<dyn Command<K>>>);
}

/// Read-only view of a UiStore used by optimizers.
pub trait UiStoreRead<K>
where
    K: Copy + Eq,
{
    /// Whether the element identified by `key` currently exists.
    fn exists(&self, key: K) -> bool;
}

pub trait MetaProvider<K, M>
where
    K: Copy + Eq,
{
    fn get_meta(&self, key: K) -> Option<M>;
}

pub trait ChildrenProvider<K, I>
where
    K: Copy + Eq,
    I: IntoIterator<Item = K>,
{
    fn children_of(&self, parent: K) -> I;
}

/// Optimize/prune/merge a batch of commands given a read-only store view.
pub trait CommandOptimizer<K>: Send + Sync {
    /// Optimizes/prunes a batch of commands.
    ///
    /// Receives ownership of `cmds` and a read-only view of the store for
    /// conservative checks. Returns a possibly shorter / merged / reordered
    /// command list that is safe to apply and semantically equivalent.
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<K>>>,
        store: &dyn UiStoreRead<K>,
    ) -> Vec<Box<dyn Command<K>>>;
}

/// Default optimizer that returns commands unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopOptimizer;

impl<K> CommandOptimizer<K> for NoopOptimizer {
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<K>>>,
        store: &dyn UiStoreRead<K>,
    ) -> Vec<Box<dyn Command<K>>> {
        let _ = store;
        cmds
    }
}

pub trait UiStore<K>: Send {
    /// Submits a command from the main-thread (may avoid synchronization).
    fn submit_from_main(&mut self, cmd: Box<dyn Command<K>>);

    /// Submits a command from other threads / systems. The store may choose
    /// to push this into an internal thread-safe queue.
    fn submit_from_other(&self, cmd: Box<dyn Command<K>>);

    /// Drains both main and external queues into the provided vector. This
    /// method should be called by the owner at a single, well-defined
    /// point (e.g. at frame start) so `&mut self` is available to apply.
    fn drain_pending(&mut self, out: &mut Vec<Box<dyn Command<K>>>);

    /// Applies a batch of commands (after optional optimization). The store
    /// implementation performs the actual mutations here under `&mut self`.
    fn apply_batch(&mut self, cmds: Vec<Box<dyn Command<K>>>);

    /// Provides a read-only snapshot view for the optimizer.
    fn as_read(&self) -> Box<dyn UiStoreRead<K> + '_>;
}

/// Lightweight handle for submitting commands.
#[derive(Clone)]
pub struct UiHandle<K> {
    queue: Arc<dyn CommandQueue<K>>,
}

impl<K> Debug for UiHandle<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiHandle")
            .field("queue", &"<command-queue>")
            .finish()
    }
}

impl<K> UiHandle<K>
where
    K: GenerationalKey + 'static,
{
    /// Creates a new handle from a boxed queue implementation.
    pub fn new(queue: Arc<dyn CommandQueue<K>>) -> Self {
        Self { queue }
    }

    /// Submits a command via the shared queue.
    pub fn submit(&self, cmd: Box<dyn Command<K>>) {
        self.queue.submit(cmd);
    }
}

/// Optional coordinator trait.
pub trait UiCoordinator<K>: Send {
    /// Drain pending commands, call optimizer and apply the final batch.
    fn flush_frame(&mut self);
}

/// Runs the canonical drain -> optimize -> apply pipeline once.
///
/// This helper drains pending commands, runs the optimizer and applies the
/// resulting batch on `store`.
pub fn run_pipeline<K, S, Q, O>(store: &mut S, queue: &Q, optimizer: &O)
where
    K: Copy + GenerationalKey + 'static,
    S: UiStore<K>,
    Q: CommandQueue<K>,
    O: CommandOptimizer<K>,
{
    let mut pending: Vec<Box<dyn Command<K>>> = Vec::new();

    store.drain_pending(&mut pending);
    queue.drain_into(&mut pending);

    // Obtain a read-only snapshot and run optimizer
    let snapshot = store.as_read();
    let optimized = optimizer.optimize(pending, snapshot.as_ref());
    drop(snapshot);

    store.apply_batch(optimized);
}
