use rimecraft_event::Event;

use super::Item;

pub static POST_PROCESS_NBT: Event<dyn Fn(Item, &mut rimecraft_nbt_ext::Compound)> =
    Event::new(|listeners| {
        Box::new(move |item, nbt| {
            for listener in listeners {
                listener(item, nbt)
            }
        })
    });
