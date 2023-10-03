use crate::registry::Registration;

/// Core block events for perform item actions and obtain block settings.
pub static EVENTS: parking_lot::RwLock<Events> = parking_lot::RwLock::new(Events(Vec::new()));

/// Manager for block events.
pub struct Events(Vec<(Option<usize>, Callback)>);

impl Events {
    /// Register a callback into this instance.
    ///
    /// The required `item` can be `None` for some events
    /// so that all items will be affected by this callback.
    pub fn register(&mut self, item: Option<super::Block>, callback: Callback) {
        self.0.push((item.map(|e| e.index_of()), callback));
    }

    pub fn block_item_map(&self, state: &super::BlockState) -> crate::item::ItemStack {
        let id = state.block().index_of();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id) && matches!(e.1, Callback::BlockStateItemMap(_))
            })
            .map_or_else(crate::item::ItemStack::default, |e| match &e.1 {
                Callback::BlockStateItemMap(c) => c(state),
                _ => unreachable!(),
            })
    }

    pub fn is_air(&self, state: &super::BlockState) -> bool {
        let id = state.block().index_of();

        self.0
            .iter()
            .find(|e| e.0.map_or(false, |ee| ee == id) && matches!(e.1, Callback::IsAir(_)))
            .map_or(false, |e| match &e.1 {
                Callback::IsAir(c) => c(state),
                _ => unreachable!(),
            })
    }

    pub fn has_random_ticks(&self, state: &super::BlockState) -> bool {
        let id = state.block().index_of();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id) && matches!(e.1, Callback::HasRandomTicks(_))
            })
            .map_or(false, |e| match &e.1 {
                Callback::HasRandomTicks(c) => c(state),
                _ => unreachable!(),
            })
    }
}

/// An block event callback variant.
pub enum Callback {
    BlockStateItemMap(fn(&super::BlockState) -> crate::item::ItemStack),
    IsAir(fn(&super::BlockState) -> bool),
    HasRandomTicks(fn(&super::BlockState) -> bool),
}
