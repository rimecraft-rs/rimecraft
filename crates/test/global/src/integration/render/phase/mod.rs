pub mod layering;
pub mod texturing;
pub mod toggleable;

pub trait Phase<'p> {
    type Handle: PhaseHandle<'p>;

    fn name(&self) -> &'static str;

    fn begin(&'p mut self) -> Self::Handle;
}

pub trait PhaseHandle<'p> {
    fn name(&'p self) -> &'static str;

    fn end(&'p mut self);
}

mod sealed {
    use super::*;

    pub trait SealedPhase<'p>: Phase<'p> {
        fn end(&mut self);
    }

    pub trait SealedPhaseHandle<'p> {
        type P: SealedPhase<'p>;

        fn phase(&'p self) -> &'p Self::P;

        fn phase_mut(&'p mut self) -> &'p mut Self::P;
    }

    impl<'p, T> PhaseHandle<'p> for T
    where
        T: SealedPhaseHandle<'p>,
        T::P: SealedPhase<'p>,
    {
        fn name(&'p self) -> &'static str {
            self.phase().name()
        }

        fn end(&'p mut self) {
            self.phase_mut().end()
        }
    }
}
