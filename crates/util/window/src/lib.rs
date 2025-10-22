//! Window management.

use rimecraft_global_cx::GlobalContext;

pub trait ProvideWindowCx: GlobalContext {
    type Window: Window;
}

pub trait WindowCx: GlobalContext + ProvideWindowCx {
    fn window(&self) -> &Self::Window;

    fn window_mut(&mut self) -> &mut Self::Window;
}

/// The position of the window on the screen, in pixels.
///
/// Supports mutual conversions with `(i32, i32)` and [`glam::IVec2`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowPos {
    /// The x position of the window.
    pub x: i32,
    /// The y position of the window.
    pub y: i32,
}

impl WindowPos {
    /// Creates a new [`WindowPos`].
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns `true` if both coordinates are zero.
    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    /// Returns `true` if both coordinates are non-zero.
    pub fn is_valid(&self) -> bool {
        self.x != 0 && self.y != 0
    }
}

impl From<(i32, i32)> for WindowPos {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl From<WindowPos> for (i32, i32) {
    fn from(pos: WindowPos) -> Self {
        (pos.x, pos.y)
    }
}

impl From<glam::IVec2> for WindowPos {
    fn from(vec: glam::IVec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl From<WindowPos> for glam::IVec2 {
    fn from(pos: WindowPos) -> Self {
        Self::new(pos.x, pos.y)
    }
}

/// The size of the window, in pixels.
///
/// Supports mutual conversions with `(u32, u32)` and [`glam::UVec2`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowSize {
    /// The width of the window.
    pub width: u32,
    /// The height of the window.
    pub height: u32,
}

impl WindowSize {
    /// Creates a new [`WindowSize`].
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns `true` if either dimension is zero.
    pub fn is_zero(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Returns `true` if both dimensions are non-negative.
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

impl From<(u32, u32)> for WindowSize {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<WindowSize> for (u32, u32) {
    fn from(size: WindowSize) -> Self {
        (size.width, size.height)
    }
}

impl From<glam::UVec2> for WindowSize {
    fn from(vec: glam::UVec2) -> Self {
        Self {
            width: vec.x,
            height: vec.y,
        }
    }
}

impl From<WindowSize> for glam::UVec2 {
    fn from(size: WindowSize) -> Self {
        Self::new(size.width, size.height)
    }
}

/// Represents a window on the screen.
pub trait Window {
    /// Whether the window is currently focused.
    fn is_focused(&self) -> bool;

    /// Whether the window is currently visible.
    fn is_visible(&self) -> bool;

    /// Whether the window is currently fullscreen.
    fn is_fullscreen(&self) -> bool;

    /// Whether the window is currently minimized.
    fn is_minimized(&self) -> bool;

    /// The size of the window, in pixels.
    fn size(&self) -> WindowSize;

    /// The scaled size of the window, in pixels.
    fn scaled_size(&self) -> WindowSize;

    /// The position of the window on the screen, in pixels.
    fn pos(&self) -> WindowPos;

    /// The scale factor of the window.
    fn scale_factor(&self) -> u32;

    /// Sets whether the window is fullscreen.
    fn set_fullscreen(&mut self, fullscreen: bool);

    /// Toggles the fullscreen state of the window.
    fn toggle_fullscreen(&mut self) {
        let is_fullscreen = self.is_fullscreen();
        self.set_fullscreen(!is_fullscreen);
    }

    /// Sets the title of the window.
    fn set_title(&mut self, title: &str);

    /// Scales the window by the given factor.
    ///
    /// This should affect the scaled size returned by [`Window::scaled_size`], and the scale factor returned by [`Window::scale_factor`].
    fn scale(&mut self, factor: u32);
}
