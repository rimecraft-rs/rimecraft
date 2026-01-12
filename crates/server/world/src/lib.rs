//! Minecraft server world implementation.

pub mod behave;
mod callback_impl;
pub mod chunk;
pub mod game_event;

pub use callback_impl::*;

//TODO: PLACEHOLDERS

/// Placeholder of type `ServerWorld`.
pub type ServerWorld<'w, Cx> = placeholder::ServerWorld<'w, Cx>;

#[allow(missing_debug_implementations)]
mod placeholder {
    use std::marker::PhantomData;

    use rcutil::{Invariant, InvariantLifetime};
    use world::WorldMarker;

    pub struct ServerWorld<'w, Cx>(PhantomData<(Cx, InvariantLifetime<'w>)>);

    impl<Cx> WorldMarker for ServerWorld<'_, Cx> {}

    unsafe impl<'w, Cx> Invariant for ServerWorld<'w, Cx> {
        type Lifetime = InvariantLifetime<'w>;
    }
}
