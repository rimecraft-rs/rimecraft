//! Manages rendering in phases.

pub trait Phase {
    type Handler: PhaseHandler;

    fn name(&self) -> &'static str;

    fn begin(&mut self) -> Self::Handler;
}

pub trait PhaseHandler {
    fn end(&mut self);
}

mod sealed {
    use super::*;
    pub struct Layering {
        pub name: &'static str,
        pub begin_action: Box<dyn FnMut()>,
        pub end_action: Box<dyn FnMut()>,
    }

    pub struct LayeringHandler {
        pub name: &'static str,
        pub end_action: Box<dyn FnMut()>,
    }

    impl Phase for Layering {
        type Handler = LayeringHandler;

        fn name(&self) -> &'static str {
            self.name
        }

        fn begin(&mut self) -> Self::Handler {
            (self.begin_action)();
            LayeringHandler {
                name: self.name,
                end_action: std::mem::replace(&mut self.end_action, Box::new(|| {})),
            }
        }
    }

    impl PhaseHandler for LayeringHandler {
        fn end(&mut self) {
            (self.end_action)();
        }
    }
}
