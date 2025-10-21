//! Drawing utilities for the rendering system.

use rimecraft_global_cx::GlobalContext;
use rimecraft_render_math::{matrix::MatrixStack, screen::ScreenPos};

pub trait ProvideDrawCx: GlobalContext {
    type Context: DrawContext;
}

pub trait DrawContext {
    fn matrices(&self) -> &MatrixStack<glam::Affine2>;

    fn matrices_mut(&mut self) -> &mut MatrixStack<glam::Affine2>;
}

pub trait Drawable<Cx>
where
    Cx: ProvideDrawCx,
{
    fn draw(&self, context: &Cx::Context, mouse_pos: impl Into<ScreenPos>, delta: f32);
}
