use std::fmt::Debug;

use super::{sealed::*, *};

pub struct Toggleable<'p, C> {
    name: &'static str,
    action: Box<dyn FnMut(C) + 'p>,
    begin_condition: C,
    end_condition: C,
}

impl<C> Debug for Toggleable<'_, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggleable")
            .field("name", &self.name)
            .field("begin_condition", &self.begin_condition)
            .field("end_condition", &self.end_condition)
            .finish()
    }
}

impl<'p, C> Phase<'p> for Toggleable<'p, C>
where
    C: Clone + 'p,
{
    type Handle = ToggleableHandle<'p, C>;

    fn name(&self) -> &'static str {
        self.name
    }

    fn begin(&'p mut self) -> Self::Handle {
        (self.action)(self.begin_condition.clone());
        ToggleableHandle(self)
    }
}

impl<'p, C> SealedPhase<'p> for Toggleable<'p, C>
where
    C: Clone + 'p,
{
    fn end(&mut self) {
        (self.action)(self.end_condition.clone());
    }
}

impl<'p, C> Toggleable<'p, C> {
    pub fn new<F>(name: &'static str, action: F, begin_condition: C, end_condition: C) -> Self
    where
        F: FnMut(C) + 'p,
    {
        Toggleable {
            name,
            action: Box::new(action),
            begin_condition,
            end_condition,
        }
    }
}

pub struct ToggleableHandle<'p, C>(&'p mut Toggleable<'p, C>);

impl<C> Debug for ToggleableHandle<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToggleableHandle")
            .field("name", &self.0.name)
            .finish()
    }
}

impl<'p, C> SealedPhaseHandle<'p> for ToggleableHandle<'p, C>
where
    C: Clone + 'p,
{
    type P = Toggleable<'p, C>;

    fn phase(&'p self) -> &'p Self::P {
        self.0
    }

    fn phase_mut(&'p mut self) -> &'p mut Self::P {
        self.0
    }
}

pub fn lightmap<'p, F>(condition: bool) -> Toggleable<'p, bool> {
    Toggleable::new(
        "lightmap",
        |_condition: bool| unimplemented!(),
        condition,
        !condition,
    )
}

pub fn overlay<'p, F>(condition: bool) -> Toggleable<'p, bool> {
    Toggleable::new(
        "overlay",
        |_condition: bool| unimplemented!(),
        condition,
        !condition,
    )
}
