use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::util::event::Event;

// pub static INITIALIZE: Lazy<Mutex<Event<(), ()>>> = Lazy::new(|| Mutex::new(Event::new(invoker, empty_impl, phases)));
