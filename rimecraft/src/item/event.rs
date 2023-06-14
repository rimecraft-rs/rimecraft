pub static EVENTS: once_cell::sync::Lazy<tokio::sync::RwLock<VanillaItemEvents>> =
    once_cell::sync::Lazy::new(|| tokio::sync::RwLock::new(VanillaItemEvents(Vec::new())));

pub struct VanillaItemEvents(Vec<(Option<usize>, VanillaItemCallback)>);

impl VanillaItemEvents {
    pub fn register(&mut self, item: Option<super::Item>, callback: VanillaItemCallback) {
        self.0.push((item.map(|e| e.id()), callback));
    }

    pub fn get_max_damage(&self, stack: &super::ItemStack) -> u32 {
        let id = stack.item.id();
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
        let id = stack.item.id();
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
        let id = item.id();
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

pub enum VanillaItemCallback {
    GetMaxCount(Box<dyn Fn(&super::ItemStack) -> u8 + 'static + Send + Sync>),
    GetMaxDamage(Box<dyn Fn(&super::ItemStack) -> u32 + 'static + Send + Sync>),
    PostProcessNbt(Box<dyn Fn(&mut crate::nbt::NbtCompound) + 'static + Send + Sync>),
}
