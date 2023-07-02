use crate::registry::Registration;

/// Vanilla item events for perform item actions and obtain item settings.
pub static EVENTS: parking_lot::RwLock<VanillaItemEvents> =
    parking_lot::RwLock::new(VanillaItemEvents(vec![]));

/// Manager for item events.
pub struct VanillaItemEvents(Vec<(Option<usize>, VanillaItemCallback)>);

impl VanillaItemEvents {
    /// Register a callback into this instance.
    ///
    /// The required `item` can be `None` for some events
    /// so that all items will be affected by this callback.
    pub fn register(&mut self, item: Option<super::Item>, callback: VanillaItemCallback) {
        self.0.push((item.map(|e| e.raw_id()), callback));
    }

    pub fn get_max_damage(&self, stack: &super::ItemStack) -> u32 {
        let id = stack.item.raw_id();
        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id)
                    && matches!(e.1, VanillaItemCallback::GetMaxDamage(_))
            })
            .map_or(0, |e| match &e.1 {
                VanillaItemCallback::GetMaxDamage(c) => c(stack),
                _ => unreachable!(),
            })
    }

    pub fn get_max_count(&self, stack: &super::ItemStack) -> u8 {
        let id = stack.item.raw_id();
        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id)
                    && matches!(e.1, VanillaItemCallback::GetMaxCount(_))
            })
            .map_or(64, |e| match &e.1 {
                VanillaItemCallback::GetMaxCount(c) => c(stack),
                _ => unreachable!(),
            })
    }

    pub fn post_process_nbt(&self, item: super::Item, nbt: &mut crate::nbt::NbtCompound) {
        let id = item.raw_id();
        self.0
            .iter()
            .filter(|e| {
                e.0.map_or(true, |ee| ee == id)
                    && matches!(e.1, VanillaItemCallback::PostProcessNbt(_))
            })
            .for_each(|e| match &e.1 {
                VanillaItemCallback::PostProcessNbt(c) => c(nbt),
                _ => unreachable!(),
            })
    }
}

/// An item event callback variant.
pub enum VanillaItemCallback {
    GetMaxCount(Box<dyn Fn(&super::ItemStack) -> u8 + 'static + Send + Sync>),
    GetMaxDamage(Box<dyn Fn(&super::ItemStack) -> u32 + 'static + Send + Sync>),
    PostProcessNbt(Box<dyn Fn(&mut crate::nbt::NbtCompound) + 'static + Send + Sync>),
}
