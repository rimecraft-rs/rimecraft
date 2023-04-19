use std::sync::atomic::{AtomicBool, Ordering};

pub mod events {
    use crate::util::{
        event::{self, Event},
        Identifier,
    };
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    pub static INITIALIZE: Lazy<Mutex<Event<(), ()>>> = Lazy::new(|| {
        Mutex::new(Event::new(
            |c, _| {
                for call in c {
                    call(())
                }
            },
            |_| (),
            vec![
                event::default_phase(),
                Identifier::parse("final".to_string()).unwrap(),
                Identifier::parse("freeze".to_string()).unwrap(),
                Identifier::parse("end".to_string()).unwrap(),
            ],
        ))
    });
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn initialize() {
    if INITIALIZED.load(Ordering::Relaxed) {
        unreachable!()
    }
    INITIALIZED.store(false, Ordering::Relaxed);
    events::INITIALIZE.lock().unwrap().register_default(|_| ());
    // TODO: registries
}
