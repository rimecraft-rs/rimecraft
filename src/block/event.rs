use crate::registry::Registration;

/// Core block events for perform item actions and obtain block settings.
pub static EVENTS: parking_lot::RwLock<CoreBlockEvents> =
    parking_lot::RwLock::new(CoreBlockEvents(Vec::new()));

/// Manager for block events.
pub struct CoreBlockEvents(Vec<(Option<usize>, CoreBlockCallback)>);

impl CoreBlockEvents {
    /// Register a callback into this instance.
    ///
    /// The required `item` can be `None` for some events
    /// so that all items will be affected by this callback.
    pub fn register(&mut self, item: Option<super::Block>, callback: CoreBlockCallback) {
        self.0.push((item.map(|e| e.raw_id()), callback));
    }

    pub fn block_item_map(&self, state: &super::BlockState) -> crate::item::ItemStack {
        let id = state.block().raw_id();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id)
                    && matches!(e.1, CoreBlockCallback::BlockStateItemMap(_))
            })
            .map_or_else(
                || crate::item::ItemStack::default(),
                |e| match &e.1 {
                    CoreBlockCallback::BlockStateItemMap(c) => c(state),
                    _ => unreachable!(),
                },
            )
    }

    pub fn is_air(&self, state: &super::BlockState) -> bool {
        let id = state.block().raw_id();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id) && matches!(e.1, CoreBlockCallback::IsAir(_))
            })
            .map_or(false, |e| match &e.1 {
                CoreBlockCallback::IsAir(c) => c(state),
                _ => unreachable!(),
            })
    }

    pub fn has_random_ticks(&self, state: &super::BlockState) -> bool {
        let id = state.block().raw_id();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id)
                    && matches!(e.1, CoreBlockCallback::HasRandomTicks(_))
            })
            .map_or(false, |e| match &e.1 {
                CoreBlockCallback::HasRandomTicks(c) => c(state),
                _ => unreachable!(),
            })
    }
}

/// An block event callback variant.
pub enum CoreBlockCallback {
    BlockStateItemMap(fn(&super::BlockState) -> crate::item::ItemStack),
    IsAir(fn(&super::BlockState) -> bool),
    HasRandomTicks(fn(&super::BlockState) -> bool),
}
