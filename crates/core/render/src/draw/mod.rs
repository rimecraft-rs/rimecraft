//! Drawing utilities for the rendering system.

use rimecraft_global_cx::GlobalContext;
use rimecraft_render_math::{
    matrix::MatrixStack,
    screen::{ScreenPos, ScreenRect},
};

pub trait ProvideDrawTy: GlobalContext {
    type Context: DrawContext;
}

pub trait DrawContext: Send + Sync {
    /// Read access to the matrix transform stack.
    fn matrices(&self) -> std::sync::RwLockReadGuard<'_, MatrixStack<glam::Affine3A>>;

    /// Write access to the matrix transform stack.
    fn matrices_mut(&self) -> std::sync::RwLockWriteGuard<'_, MatrixStack<glam::Affine3A>>;

    /// Read access to the scissor rectangle stack.
    fn scissors(&self) -> std::sync::RwLockReadGuard<'_, MatrixStack<ScreenRect>>;

    /// Write access to the scissor rectangle stack.
    fn scissors_mut(&self) -> std::sync::RwLockWriteGuard<'_, MatrixStack<ScreenRect>>;
}

pub trait Drawable<Cx>
where
    Cx: ProvideDrawTy,
{
    fn draw(&self, context: &Cx::Context, mouse_pos: impl Into<ScreenPos>, delta: f32);
}
