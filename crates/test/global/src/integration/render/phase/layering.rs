use std::fmt::Debug;

use super::{sealed::*, *};

pub struct Layering<'p> {
    name: &'static str,
    begin_action: Box<dyn FnMut() + 'p>,
    end_action: Box<dyn FnMut() + 'p>,
}

impl Debug for Layering<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layering")
            .field("name", &self.name)
            .finish()
    }
}

impl<'p> Phase<'p> for Layering<'p> {
    type Handle = LayeringHandle<'p>;

    fn name(&self) -> &'static str {
        self.name
    }

    fn begin(&'p mut self) -> Self::Handle {
        (self.begin_action)();
        LayeringHandle(self)
    }
}

impl<'p> SealedPhase<'p> for Layering<'p> {
    fn end(&mut self) {
        (self.end_action)();
    }
}

impl<'p> Layering<'p> {
    pub fn new<F1, F2>(name: &'static str, begin_action: F1, end_action: F2) -> Self
    where
        F1: FnMut() + 'p,
        F2: FnMut() + 'p,
    {
        Layering {
            name,
            begin_action: Box::new(begin_action),
            end_action: Box::new(end_action),
        }
    }
}

pub struct LayeringHandle<'p>(&'p mut Layering<'p>);

impl Debug for LayeringHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayeringHandle")
            .field("name", &self.0.name)
            .finish()
    }
}

impl<'p> SealedPhaseHandle<'p> for LayeringHandle<'p> {
    type P = Layering<'p>;

    fn phase(&'p self) -> &'p Self::P {
        self.0
    }

    fn phase_mut(&'p mut self) -> &'p mut Self::P {
        self.0
    }
}
