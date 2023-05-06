use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

pub mod events {
    use crate::util::{
        event::{self, Event},
        Identifier,
    };
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    pub static INITIALIZE: Lazy<Mutex<Event<(), ()>>> = Lazy::new(|| {
        Mutex::new(Event::new(
            Box::new(|c, _| {
                for call in c {
                    call(())
                }
            }),
            Box::new(|_| ()),
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
pub static LOAD_TIME: AtomicU64 = AtomicU64::new(0);

pub fn initialize() {
    if INITIALIZED.load(Ordering::Relaxed) {
        return;
    }
    INITIALIZED.store(false, Ordering::Relaxed);
    let instant = Instant::now();
    // TODO: registries
    LOAD_TIME.store(instant.elapsed().as_millis() as u64, Ordering::Relaxed);
}
