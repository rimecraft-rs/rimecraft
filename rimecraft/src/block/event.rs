use crate::registry::Registration;

/// Vanilla block events for perform item actions and obtain block settings.
pub static EVENTS: parking_lot::RwLock<VanillaBlockEvents> =
    parking_lot::RwLock::new(VanillaBlockEvents(Vec::new()));

/// Manager for block events.
pub struct VanillaBlockEvents(Vec<(Option<usize>, VanillaBlockCallback)>);

impl VanillaBlockEvents {
    /// Register a callback into this instance.
    ///
    /// The required `item` can be `None` for some events
    /// so that all items will be affected by this callback.
    pub fn register(&mut self, item: Option<super::Block>, callback: VanillaBlockCallback) {
        self.0.push((item.map(|e| e.raw_id()), callback));
    }

    pub fn block_item_map(&self, state: super::BlockState) -> crate::item::ItemStack {
        let id = state.block().raw_id();
        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id)
                    && matches!(e.1, VanillaBlockCallback::BlockStateItemMap(_))
            })
            .map_or_else(
                || crate::item::ItemStack::default(),
                |e| match &e.1 {
                    VanillaBlockCallback::BlockStateItemMap(c) => c(state),
                    _ => unreachable!(),
                },
            )
    }
}

/// An item event callback variant.
pub enum VanillaBlockCallback {
    BlockStateItemMap(
        Box<dyn Fn(super::BlockState) -> crate::item::ItemStack + 'static + Send + Sync>,
    ),
}
