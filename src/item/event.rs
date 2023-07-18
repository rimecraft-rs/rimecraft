use crate::registry::Registration;

/// Core item events for perform item actions and obtain item settings.
pub static EVENTS: parking_lot::RwLock<CoreItemEvents> =
    parking_lot::RwLock::new(CoreItemEvents(vec![]));

/// Manager for item events.
pub struct CoreItemEvents(Vec<(Option<usize>, CoreItemCallback)>);

impl CoreItemEvents {
    /// Register a callback into this instance.
    ///
    /// The required `item` can be `None` for some events
    /// so that all items will be affected by this callback.
    pub fn register(&mut self, item: Option<super::Item>, callback: CoreItemCallback) {
        self.0.push((item.map(|e| e.raw_id()), callback));
    }

    pub fn get_max_damage(&self, stack: &super::ItemStack) -> u32 {
        let id = stack.item.raw_id();
        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id) && matches!(e.1, CoreItemCallback::GetMaxDamage(_))
            })
            .map_or(0, |e| match &e.1 {
                CoreItemCallback::GetMaxDamage(c) => c(stack),
                _ => unreachable!(),
            })
    }

    pub fn get_max_count(&self, stack: &super::ItemStack) -> u8 {
        let id = stack.item.raw_id();
        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id) && matches!(e.1, CoreItemCallback::GetMaxCount(_))
            })
            .map_or(64, |e| match &e.1 {
                CoreItemCallback::GetMaxCount(c) => c(stack),
                _ => unreachable!(),
            })
    }

    pub fn post_process_nbt(&self, item: super::Item, nbt: &mut crate::nbt::NbtCompound) {
        let id = item.raw_id();
        self.0
            .iter()
            .filter(|e| {
                e.0.map_or(true, |ee| ee == id)
                    && matches!(e.1, CoreItemCallback::PostProcessNbt(_))
            })
            .for_each(|e| match &e.1 {
                CoreItemCallback::PostProcessNbt(c) => c(nbt),
                _ => unreachable!(),
            })
    }
}

/// An item event callback variant.
pub enum CoreItemCallback {
    GetMaxCount(fn(&super::ItemStack) -> u8),
    GetMaxDamage(fn(&super::ItemStack) -> u32),
    PostProcessNbt(fn(&mut crate::nbt::NbtCompound)),
}
