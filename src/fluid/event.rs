use crate::registry::Registration;

/// Core fluid events for perform fluid actions and obtain fluid settings.
pub static EVENTS: parking_lot::RwLock<CoreFluidEvents> =
    parking_lot::RwLock::new(CoreFluidEvents(Vec::new()));

/// Manager for fluid events.
pub struct CoreFluidEvents(Vec<(Option<usize>, CoreFluidCallback)>);

impl CoreFluidEvents {
    /// Register a callback into this instance.
    ///
    /// The required `item` can be `None` for some events
    /// so that all fluids will be affected by this callback.
    pub fn register(&mut self, item: Option<super::Fluid>, callback: CoreFluidCallback) {
        self.0.push((item.map(|e| e.raw_id()), callback));
    }

    pub fn is_empty(&self, state: &super::FluidState) -> bool {
        let id = state.fluid().raw_id();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id) && matches!(e.1, CoreFluidCallback::IsEmpty(_))
            })
            .map_or(false, |e| match &e.1 {
                CoreFluidCallback::IsEmpty(c) => c(state),
                _ => unreachable!(),
            })
    }

    pub fn has_random_ticks(&self, state: &super::FluidState) -> bool {
        let id = state.fluid().raw_id();

        self.0
            .iter()
            .find(|e| {
                e.0.map_or(false, |ee| ee == id)
                    && matches!(e.1, CoreFluidCallback::HasRandomTicks(_))
            })
            .map_or(false, |e| match &e.1 {
                CoreFluidCallback::HasRandomTicks(c) => c(state),
                _ => unreachable!(),
            })
    }
}

/// An block event callback variant.
pub enum CoreFluidCallback {
    IsEmpty(fn(&super::FluidState) -> bool),
    HasRandomTicks(fn(&super::FluidState) -> bool),
}
