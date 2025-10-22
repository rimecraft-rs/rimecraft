//! Drawing utilities for the rendering system.

use rimecraft_global_cx::GlobalContext;
use rimecraft_render_math::{
    matrix::MatrixStack,
    screen::{ScreenPos, ScreenRect},
};

/// Provides type information for drawing.
pub trait ProvideDrawTy: GlobalContext {
    /// The drawing context.
    type Context: DrawContext;
}

/// The drawing context passed to drawable objects.
pub trait DrawContext: Send + Sync {
    /// Associated type for a read guard over the matrices stack.
    type MatricesReadGuard<'a>: std::ops::Deref<Target = MatrixStack<glam::Affine3A>> + 'a
    where
        Self: 'a;

    /// Associated type for a write guard over the matrices stack.
    type MatricesWriteGuard<'a>: std::ops::DerefMut<Target = MatrixStack<glam::Affine3A>> + 'a
    where
        Self: 'a;

    /// Associated type for a read guard over the scissors stack.
    type ScissorsReadGuard<'a>: std::ops::Deref<Target = MatrixStack<ScreenRect>> + 'a
    where
        Self: 'a;

    /// Associated type for a write guard over the scissors stack.
    type ScissorsWriteGuard<'a>: std::ops::DerefMut<Target = MatrixStack<ScreenRect>> + 'a
    where
        Self: 'a;

    /// Read access to the matrix transform stack.
    fn matrices(&self) -> Self::MatricesReadGuard<'_>;

    /// Write access to the matrix transform stack.
    fn matrices_mut(&self) -> Self::MatricesWriteGuard<'_>;

    /// Read access to the scissor rectangle stack.
    fn scissors(&self) -> Self::ScissorsReadGuard<'_>;

    /// Write access to the scissor rectangle stack.
    fn scissors_mut(&self) -> Self::ScissorsWriteGuard<'_>;
}

pub trait Drawable<Cx>
where
    Cx: ProvideDrawTy,
{
    fn draw(&self, context: &Cx::Context, mouse_pos: impl Into<ScreenPos>, delta: f32);
}
