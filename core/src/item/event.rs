use crate::util::Event;

use super::Item;

pub static POST_PROCESS_NBT: Event<dyn Fn(Item, &mut crate::nbt::NbtCompound)> =
    Event::new(|listeners| {
        Box::new(move |item, nbt| {
            for listener in listeners {
                listener(item, nbt)
            }
        })
    });
