//! Minecraft client UI framework.

use std::collections::VecDeque;

use crate::screen::ScreenRect;

pub mod context;
pub mod nav;
pub mod render;
pub mod screen;

pub trait Drawable {
    fn draw(&self, context: (), mouse_pos: glam::Vec2);
}

pub trait ScissorStackOp {
    fn current(&self) -> Option<&ScreenRect>;

    fn contains(&self, point: glam::Vec2) -> bool {
        if let Some(rect) = self.current() {
            rect.contains_pos(point.into())
        } else {
            true
        }
    }

    fn enter<'a>(&'a mut self, rect: ScreenRect) -> ScissorStackHandle<'a>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ScissorStack {
    stack: VecDeque<ScreenRect>,
}

impl ScissorStack {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    fn leave(&mut self) {
        self.stack.pop_back();
    }
}

impl ScissorStackOp for ScissorStack {
    fn current(&self) -> Option<&ScreenRect> {
        self.stack.back()
    }

    fn enter<'a>(&'a mut self, rect: ScreenRect) -> ScissorStackHandle<'a> {
        self.stack.push_back(rect);
        ScissorStackHandle(self)
    }
}

pub struct ScissorStackHandle<'a>(&'a mut ScissorStack);

impl Drop for ScissorStackHandle<'_> {
    fn drop(&mut self) {
        self.0.leave();
    }
}

impl ScissorStackOp for ScissorStackHandle<'_> {
    fn current(&self) -> Option<&ScreenRect> {
        self.0.current()
    }

    fn enter<'a>(&'a mut self, rect: ScreenRect) -> ScissorStackHandle<'a> {
        self.0.enter(rect)
    }
}
