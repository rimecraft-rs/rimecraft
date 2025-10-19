//! Minecraft client UI framework.

pub mod context;
pub mod nav;
pub mod screen;

pub trait Drawable {
    fn draw(&self, context: (), mouse_pos: glam::Vec2);
}
