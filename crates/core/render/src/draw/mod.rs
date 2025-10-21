//! Drawing utilities for the rendering system.

use rimecraft_global_cx::GlobalContext;
use rimecraft_render_math::matrix::MatrixStack;

pub trait ProvideDrawCx: GlobalContext {
    type DrawExt;
}

pub struct DrawContext {
    pub matrices: MatrixStack<glam::Affine2>,
}

pub trait Drawable {
    fn draw(&self, context: &DrawContext, mouse_pos: glam::Vec2);
}
