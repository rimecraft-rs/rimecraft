//! Navigate through the GUI hierarchy.

use crate::{
    Element,
    nav::{NavAxis, NavDirection},
};

/// A navigation action in the GUI.
pub trait GuiNavigation {
    /// The direction of the navigation.
    fn direction(&self) -> NavDirection;
}

/// A navigation action that always goes down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuiDownNavigation;

impl GuiNavigation for GuiDownNavigation {
    fn direction(&self) -> NavDirection {
        NavDirection::Down // Always navigates down
    }
}

/// Navigation triggered by arrow keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuiArrowNavigation {
    pub direction: NavDirection,
}

impl GuiNavigation for GuiArrowNavigation {
    fn direction(&self) -> NavDirection {
        match self.direction.axis() {
            NavAxis::Vertical => self.direction,
            NavAxis::Horizontal => NavDirection::Down, // Horizontal arrow directions will lead to a deeper navigation level
        }
    }
}

/// Navigation triggered by Tab key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuiTabNavigation {
    /// Whether the navigation is forward (Tab) or backward (Shift+Tab).
    pub forward: bool,
}

impl GuiNavigation for GuiTabNavigation {
    fn direction(&self) -> NavDirection {
        if self.forward {
            NavDirection::Down // Tab navigates down
        } else {
            NavDirection::Up // Shift+Tab navigates up
        }
    }
}

pub trait GuiNavigationPath<Cx> {
    fn element(&self) -> &dyn Element<Cx = Cx>;

    fn set_focused(&mut self, focused: bool);

    fn focus(&mut self) {
        self.set_focused(true);
    }
}

// pub struct GuiNavigationIntermediate<Cx> {
//     pub element: Box<dyn Element<Cx = Cx>>,
//     pub focused: bool,
// }
