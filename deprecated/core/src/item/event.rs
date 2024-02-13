use std::sync::Arc;

use rimecraft_event::{DefaultSyncEvent, Event};

use super::Item;

pub static POST_PROCESS_NBT: DefaultSyncEvent<
    dyn Fn(Item, &mut rimecraft_nbt_ext::Compound) + Send + Sync,
> = Event::new(|listeners| {
    Arc::new(move |item, nbt| {
        for listener in &listeners {
            listener(item, nbt)
        }
    })
});
