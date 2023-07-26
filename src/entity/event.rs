use crate::registry::Registration;

/// Core entity events for perform entity actions and obtain entity settings.
pub static EVENTS: parking_lot::RwLock<Events> = parking_lot::RwLock::new(Events(Vec::new()));

/// Manager for entity events.
pub struct Events(Vec<(Option<usize>, Callback)>);

impl Events {
    /// Register a callback into this instance.
    ///
    /// The required `entity` can be `None` for some events
    /// so that all entities will be affected by this callback.
    pub fn register(&mut self, entity: Option<super::Type>, callback: Callback) {
        self.0.push((entity.map(|e| e.raw_id()), callback));
    }

    pub fn is_summonable(&self, entity_type: super::Type) -> bool {
        let id = entity_type.raw_id();

        self.0
            .iter()
            .find(|e| e.0.map_or(false, |ee| ee == id) && matches!(e.1, Callback::Summonable(_)))
            .map_or(false, |e| match &e.1 {
                Callback::Summonable(c) => *c,
                _ => unreachable!(),
            })
    }

    pub fn spawnable_blocks(&self, entity_type: super::Type) -> Vec<crate::block::Block> {
        let id = entity_type.raw_id();

        let mut vec = Vec::new();

        for e in self
            .0
            .iter()
            .filter(|e| e.0.map_or(true, |ee| ee == id) && matches!(e.1, Callback::Summonable(_)))
            .map(|e| match &e.1 {
                Callback::SpawnableBlocks(c) => *c,
                _ => unreachable!(),
            })
        {
            if vec.is_empty() {
                vec = e.to_vec()
            } else {
                vec.append(&mut e.iter().copied().filter(|b| !vec.contains(b)).collect());
            }
        }

        vec
    }
}

/// An entity event callback variant.
pub enum Callback {
    Summonable(bool),
    SpawnableBlocks(&'static [crate::block::Block]),
}
