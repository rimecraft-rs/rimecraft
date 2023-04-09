use once_cell::sync::Lazy;
use std::sync::Mutex;

static INITIALIZED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn initialize() {
    let mut initialized = *INITIALIZED.lock().unwrap();
    if initialized {
        unreachable!()
    }
    initialized = true;
    // TODO: registries
}
