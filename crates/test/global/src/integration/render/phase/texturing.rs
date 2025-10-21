use std::fmt::Debug;

use super::{sealed::*, *};

pub struct Texturing<'p> {
    name: &'static str,
    begin_action: Box<dyn FnMut() + 'p>,
    end_action: Box<dyn FnMut() + 'p>,
}

impl Debug for Texturing<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texturing")
            .field("name", &self.name)
            .finish()
    }
}

impl<'p> Phase<'p> for Texturing<'p> {
    type Handle = TexturingHandle<'p>;

    fn name(&self) -> &'static str {
        self.name
    }

    fn begin(&'p mut self) -> Self::Handle {
        (self.begin_action)();
        TexturingHandle(self)
    }
}

impl<'p> SealedPhase<'p> for Texturing<'p> {
    fn end(&mut self) {
        (self.end_action)();
    }
}

impl<'p> Texturing<'p> {
    pub fn new<F>(name: &'static str, begin_action: F, end_action: F) -> Self
    where
        F: FnMut() + 'p,
    {
        Texturing {
            name,
            begin_action: Box::new(begin_action),
            end_action: Box::new(end_action),
        }
    }
}

pub struct TexturingHandle<'p>(&'p mut Texturing<'p>);

impl Debug for TexturingHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TexturingHandle")
            .field("name", &self.0.name)
            .finish()
    }
}

impl<'p> SealedPhaseHandle<'p> for TexturingHandle<'p> {
    type P = Texturing<'p>;

    fn phase(&'p self) -> &'p Self::P {
        self.0
    }

    fn phase_mut(&'p mut self) -> &'p mut Self::P {
        self.0
    }
}
