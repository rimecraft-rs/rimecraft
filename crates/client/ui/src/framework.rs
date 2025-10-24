//! Minimal, implementation-agnostic primitives for a command-driven UI.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// A single UI mutation described as a value object.
pub trait Command<Id>: Send + fmt::Debug {
    /// Apply this command to `store`.
    fn apply(&self, store: &mut dyn UiStore<Id>);

    /// Downcast helper.
    fn as_any(&self) -> &dyn Any;
}

// Note: we intentionally do not provide a blanket impl for `Command` to
// avoid relying on unstable specialization. Concrete command types should
// implement `Command<Id>` explicitly.

/// Abstract queue for submitting commands.
pub trait CommandQueue<Id>: Send + Sync {
    /// Submit a command.
    fn submit(&self, cmd: Box<dyn Command<Id>>);

    /// Drain all queued commands into `out`.
    fn drain_into(&self, out: &mut Vec<Box<dyn Command<Id>>>);
}

/// Read-only view of a UiStore used by optimizers.
pub trait UiStoreRead<Id> {
    /// Whether the element identified by `id` currently exists.
    fn exists(&self, id: Id) -> bool;
}

/// Optimize/prune/merge a batch of commands given a read-only store view.
pub trait CommandOptimizer<Id>: Send + Sync {
    /// Optimizes/prunes a batch of commands.
    ///
    /// Receives ownership of `cmds` and a read-only view of the store for
    /// conservative checks. Returns a possibly shorter / merged / reordered
    /// command list that is safe to apply and semantically equivalent.
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<Id>>>,
        store: &dyn UiStoreRead<Id>,
    ) -> Vec<Box<dyn Command<Id>>>;
}

/// Default optimizer that returns commands unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopOptimizer;

impl<Id> CommandOptimizer<Id> for NoopOptimizer {
    fn optimize(
        &self,
        cmds: Vec<Box<dyn Command<Id>>>,
        store: &dyn UiStoreRead<Id>,
    ) -> Vec<Box<dyn Command<Id>>> {
        let _ = store;
        cmds
    }
}

/// Minimal write API for a UiStore.
pub trait UiStore<Id>: Send {
    /// Submits a command from the main-thread (may avoid synchronization).
    fn submit_from_main(&mut self, cmd: Box<dyn Command<Id>>);

    /// Submits a command from other threads / systems. The store may choose
    /// to push this into an internal thread-safe queue.
    fn submit_from_other(&self, cmd: Box<dyn Command<Id>>);

    /// Drains both main and external queues into the provided vector. This
    /// method should be called by the owner at a single, well-defined
    /// point (e.g. at frame start) so `&mut self` is available to apply.
    fn drain_pending(&mut self, out: &mut Vec<Box<dyn Command<Id>>>);

    /// Applies a batch of commands (after optional optimization). The store
    /// implementation performs the actual mutations here under `&mut self`.
    fn apply_batch(&mut self, cmds: Vec<Box<dyn Command<Id>>>);

    /// Provides a read-only snapshot view for the optimizer.
    fn as_read(&self) -> Box<dyn UiStoreRead<Id> + '_>;
}

/// Lightweight handle for submitting commands.
#[derive(Clone)]
pub struct UiHandle<Id> {
    queue: Arc<dyn CommandQueue<Id>>,
}

impl<Id> fmt::Debug for UiHandle<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiHandle")
            .field("queue", &"<command-queue>")
            .finish()
    }
}

impl<Id> UiHandle<Id>
where
    Id: 'static,
{
    /// Creates a new handle from a boxed queue implementation.
    pub fn new(queue: Arc<dyn CommandQueue<Id>>) -> Self {
        Self { queue }
    }

    /// Submits a command via the shared queue.
    pub fn submit(&self, cmd: Box<dyn Command<Id>>) {
        self.queue.submit(cmd);
    }
}

/// Optional coordinator trait.
pub trait UiCoordinator<Id>: Send {
    /// Drain pending commands, call optimizer and apply the final batch.
    fn flush_frame(&mut self);
}

/// Runs the canonical drain -> optimize -> apply pipeline once.
///
/// This helper drains pending commands, runs the optimizer and applies the
/// resulting batch on `store`.
pub fn run_pipeline<Id, S, Q, O>(store: &mut S, queue: &Q, optimizer: &O)
where
    Id: Copy + 'static,
    S: UiStore<Id>,
    Q: CommandQueue<Id>,
    O: CommandOptimizer<Id>,
{
    let mut pending: Vec<Box<dyn Command<Id>>> = Vec::new();

    store.drain_pending(&mut pending);
    queue.drain_into(&mut pending);

    // Obtain a read-only snapshot and run optimizer
    let snapshot = store.as_read();
    let optimized = optimizer.optimize(pending, snapshot.as_ref());
    drop(snapshot);

    store.apply_batch(optimized);
}

// End of module
