use once_cell::sync::Lazy;
use std::sync::Mutex;

static INITIALIZED: Lazy<Mutex<InitSwitch>> = Lazy::new(|| {
    Mutex::new(InitSwitch {
        initialized: bool::default(),
    })
});

struct InitSwitch {
    pub initialized: bool,
}

pub fn initialize() {
    if INITIALIZED.lock().unwrap().initialized {
        return;
    }
    INITIALIZED.lock().unwrap().initialized = true;
    // TODO: registries
}
