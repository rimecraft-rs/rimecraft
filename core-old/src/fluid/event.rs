use crate::registry::Registration;

/// Core fluid events for perform fluid actions and obtain fluid settings.
pub static EVENTS: parking_lot::RwLock<Events> = parking_lot::RwLock::new(Events(Vec::new()));

/// Manager for fluid events.
pub struct Events(Vec<(Option<usize>, Callback)>);

impl Events {
    /// Register a callback into this instance.
    ///
    /// The required `fluid` can be `None` for some events
    /// so that all fluids will be affected by this callback.
    pub fn register(&mut self, fluid: Option<super::Fluid>, callback: Callback) {
        self.0.push((fluid.map(|e| e.index_of()), callback));
    }

    pub fn is_empty(&self, state: &super::FluidState) -> bool {
        let id = state.fluid().index_of();

        self.0
            .iter()
            .find(|e| e.0.map_or(false, |ee| ee == id) && matches!(e.1, Callback::IsEmpty(_)))
            .map_or(false, |e| match &e.1 {
                Callback::IsEmpty(c) => c(state),
                _ => unreachable!(),
            })
    }

    pub fn has_random_ticks(&self, state: &super::FluidState) -> bool {
        let id = state.fluid().index_of();

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

/// An fluid event callback variant.
pub enum Callback {
    IsEmpty(fn(&super::FluidState) -> bool),
    HasRandomTicks(fn(&super::FluidState) -> bool),
}
