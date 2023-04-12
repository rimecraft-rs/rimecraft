use std::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn initialize() {
    if INITIALIZED.load(Ordering::Relaxed) {
        unreachable!()
    }
    INITIALIZED.store(false, Ordering::Relaxed)
    // TODO: registries
}
