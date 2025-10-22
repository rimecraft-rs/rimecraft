//! Navigate through the GUI hierarchy.

use crate::{
    Element, ParentElement,
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

pub struct GuiNavigationNode<Cx, E>
where
    E: ParentElement<Cx = Cx>,
{
    element: Box<E>,
    child: Box<dyn GuiNavigationPath<Cx>>,
}

impl<Cx, E> GuiNavigationPath<Cx> for GuiNavigationNode<Cx, E>
where
    E: ParentElement<Cx = Cx>,
{
    fn element(&self) -> &dyn Element<Cx = Cx> {
        self.element.as_ref()
    }

    fn set_focused(&mut self, focused: bool) {
        self.element.set_focused(false);

        if let Some(index) = self.element.child_index(self.child.element()) {
            self.element.children_mut()[index].set_focused(focused);
        }

        self.child.set_focused(focused);
    }
}

pub struct GuiNavigationLeaf<Cx, E>
where
    E: Element<Cx = Cx>,
{
    element: Box<E>,
}

impl<Cx, E> GuiNavigationPath<Cx> for GuiNavigationLeaf<Cx, E>
where
    E: Element<Cx = Cx>,
{
    fn element(&self) -> &dyn Element<Cx = Cx> {
        self.element.as_ref()
    }

    fn set_focused(&mut self, focused: bool) {
        self.element.set_focused(focused);
    }
}

pub fn leaf<Cx, E>(element: E) -> impl GuiNavigationPath<Cx>
where
    E: Element<Cx = Cx>,
{
    GuiNavigationLeaf {
        element: Box::new(element),
    }
}

pub fn node<Cx, E, Child>(element: E, child: Child) -> impl GuiNavigationPath<Cx>
where
    E: ParentElement<Cx = Cx>,
    Child: GuiNavigationPath<Cx> + 'static,
{
    GuiNavigationNode {
        element: Box::new(element),
        child: Box::new(child),
    }
}

pub fn path<Cx, E>(
    leaf: E,
    parents: impl IntoIterator<Item = Box<dyn ParentElement<Cx = Cx>>>,
) -> impl GuiNavigationPath<Cx>
where
    E: Element<Cx = Cx>,
{
    let mut current: Box<dyn GuiNavigationPath<Cx>> = Box::new(crate::nav::gui::leaf(leaf));

    for parent in parents.into_iter() {
        current = Box::new(node(parent, current));
    }

    current
}
