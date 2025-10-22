//! Navigate through the GUI hierarchy.

use std::fmt::Debug;

use rimecraft_keyboard::ProvideKeyboardTy;
use rimecraft_mouse::ProvideMouseTy;

use crate::{
    Element, Focusable, ParentElement,
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
pub struct GuiArrowNavigation(pub NavDirection);

impl GuiNavigation for GuiArrowNavigation {
    fn direction(&self) -> NavDirection {
        match self.0.axis() {
            NavAxis::Vertical => self.0,
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

// /// A path through the GUI elements for navigation.
// pub trait GuiNavigationTarget<Cx> {
//     /// The current element in the navigation path.
//     fn element(&self) -> &dyn Element<Cx>;

//     /// Sets whether this element is focused.
//     fn set_focused(&mut self, focused: bool);

//     /// Focuses this element.
//     fn focus(&mut self) {
//         self.set_focused(true);
//     }
// }

// impl<T, Cx> GuiNavigationTarget<Cx> for Box<T>
// where
//     T: GuiNavigationTarget<Cx> + ?Sized,
// {
//     #[inline(always)]
//     fn element(&self) -> &dyn Element<Cx> {
//         (**self).element()
//     }

//     #[inline(always)]
//     fn set_focused(&mut self, focused: bool) {
//         (**self).set_focused(focused)
//     }

//     #[inline(always)]
//     fn focus(&mut self) {
//         (**self).focus()
//     }
// }

#[allow(clippy::exhaustive_enums)]
pub enum GuiNavigationPath<Cx>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    Node {
        element: Box<dyn ParentElement<Cx>>,
        child: Box<GuiNavigationPath<Cx>>,
    },
    Leaf(Box<dyn Element<Cx>>),
}

impl<Cx> Debug for GuiNavigationPath<Cx>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiNavigationPath::Node { .. } => f.debug_struct("GuiNavigationPath::Node").finish(),
            GuiNavigationPath::Leaf(_) => f.debug_struct("GuiNavigationPath::Leaf").finish(),
        }
    }
}

impl<Cx> Focusable<Cx> for GuiNavigationPath<Cx>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    fn is_focused(&self) -> bool {
        match self {
            GuiNavigationPath::Node { element, child } => {
                if let Some(index) = element.child_index(child.element()) {
                    element.children()[index].is_focused()
                } else {
                    false
                }
            }
            GuiNavigationPath::Leaf(element) => element.is_focused(),
        }
    }

    fn set_focused(&mut self, focused: bool) {
        match self {
            GuiNavigationPath::Node { element, child } => {
                element.set_focused(false);

                if let Some(index) = element.child_index(child.element()) {
                    element.children_mut()[index].set_focused(focused);
                }

                child.set_focused(focused);
            }
            GuiNavigationPath::Leaf(element) => {
                element.set_focused(focused);
            }
        }
    }

    fn focus(&mut self) {
        self.set_focused(true);
    }
}

impl<Cx> GuiNavigationPath<Cx>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    pub fn element(&self) -> &dyn Element<Cx> {
        match self {
            GuiNavigationPath::Node { element, .. } => element.as_ref(),
            GuiNavigationPath::Leaf(element) => element.as_ref(),
        }
    }

    pub fn leaf<E>(element: E) -> Self
    where
        E: Element<Cx> + 'static,
        Cx: ProvideKeyboardTy + ProvideMouseTy,
    {
        GuiNavigationPath::Leaf(Box::new(element))
    }

    pub fn node<E>(element: E, child: GuiNavigationPath<Cx>) -> Self
    where
        E: ParentElement<Cx> + 'static,
        Cx: ProvideKeyboardTy + ProvideMouseTy,
    {
        GuiNavigationPath::Node {
            element: Box::new(element),
            child: Box::new(child),
        }
    }

    pub fn path<E>(leaf: E, parents: Vec<Box<dyn ParentElement<Cx>>>) -> Self
    where
        E: Element<Cx> + 'static,
        Cx: ProvideKeyboardTy + ProvideMouseTy,
    {
        let mut current = GuiNavigationPath::leaf(leaf);

        for parent in parents.into_iter().rev() {
            current = GuiNavigationPath::Node {
                element: parent,
                child: Box::new(current),
            };
        }

        current
    }
}

// /// A navigation node with a parent element and a child path.
// pub struct GuiNavigationNode<'a, Cx, E>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: ParentElement<Cx> + ?Sized,
// {
//     element: Box<E>,
//     child: Box<dyn GuiNavigationTarget<Cx> + 'a>,
// }

// impl<Cx, E> Debug for GuiNavigationNode<'_, Cx, E>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: ParentElement<Cx> + ?Sized,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("GuiNavigationNode").finish()
//     }
// }

// impl<Cx, E> GuiNavigationTarget<Cx> for GuiNavigationNode<'_, Cx, E>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: ParentElement<Cx>,
// {
//     fn element(&self) -> &dyn Element<Cx> {
//         self.element.as_ref()
//     }

//     fn set_focused(&mut self, focused: bool) {
//         self.element.set_focused(false);

//         if let Some(index) = self.element.child_index(self.child.element()) {
//             self.element.children_mut()[index].set_focused(focused);
//         }

//         self.child.set_focused(focused);
//     }
// }

// impl<'a, Cx> GuiNavigationTarget<Cx> for GuiNavigationNode<'_, Cx, dyn ParentElement<Cx> + 'a>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     dyn ParentElement<Cx> + 'a: Element<Cx>,
// {
//     fn element(&self) -> &dyn Element<Cx> {
//         self.element.as_ref()
//     }

//     fn set_focused(&mut self, focused: bool) {
//         self.element.set_focused(false);

//         if let Some(index) = self.element.child_index(self.child.element()) {
//             self.element.children_mut()[index].set_focused(focused);
//         }

//         self.child.set_focused(focused);
//     }
// }

// /// A navigation leaf with a single element.
// #[derive(Debug)]
// pub struct GuiNavigationLeaf<Cx, E>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: Element<Cx> + ?Sized,
// {
//     element: Box<E>,
//     _marker: std::marker::PhantomData<Cx>,
// }

// impl<Cx, E> GuiNavigationTarget<Cx> for GuiNavigationLeaf<Cx, E>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: Element<Cx>,
// {
//     fn element(&self) -> &dyn Element<Cx> {
//         self.element.as_ref()
//     }

//     fn set_focused(&mut self, focused: bool) {
//         self.element.set_focused(focused);
//     }
// }

// impl<Cx> GuiNavigationTarget<Cx> for GuiNavigationLeaf<Cx, dyn Element<Cx>>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     dyn Element<Cx>: Element<Cx>,
// {
//     fn element(&self) -> &dyn Element<Cx> {
//         self.element.as_ref()
//     }

//     fn set_focused(&mut self, focused: bool) {
//         self.element.set_focused(focused);
//     }
// }

// /// Creates a [`GuiNavigationLeaf`] with the given element.
// pub fn leaf<Cx, E>(element: E) -> impl GuiNavigationTarget<Cx>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: Element<Cx>,
// {
//     GuiNavigationLeaf {
//         element: Box::new(element),
//         _marker: std::marker::PhantomData,
//     }
// }

// /// Creates a [`GuiNavigationNode`] with the given parent element and child path.
// pub fn node<'a, Cx, E, Child>(element: E, child: Child) -> impl GuiNavigationTarget<Cx>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy,
//     E: ParentElement<Cx>,
//     Child: GuiNavigationTarget<Cx> + 'a,
// {
//     GuiNavigationNode {
//         element: Box::new(element),
//         child: Box::new(child),
//     }
// }

// /// Creates a full navigation path from a leaf element and an iterator of parent elements.
// pub fn path<'a, Cx, E, P>(leaf: E, parents: P) -> impl GuiNavigationTarget<Cx>
// where
//     Cx: ProvideKeyboardTy + ProvideMouseTy + 'a,
//     E: Element<Cx> + 'a,
//     P: IntoIterator<Item = Box<dyn ParentElement<Cx>>>,
//     dyn ParentElement<Cx>: Element<Cx>,
// {
//     let mut current: Box<dyn GuiNavigationTarget<Cx> + 'a> = Box::new(crate::nav::gui::leaf(leaf));

//     for parent in parents.into_iter() {
//         current = Box::new(GuiNavigationNode {
//             element: parent,
//             child: current,
//         });
//     }

//     current
// }

#[cfg(test)]
mod tests {
    use rimecraft_test_global::{TestContext, integration::mouse::TestButton};

    use crate::{Focusable, ParentElementExt, ParentElementFocusableImpl, nav::WithNavIndex};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestElement {
        name: &'static str,
        focused: bool,
    }

    impl TestElement {
        const fn new(name: &'static str) -> Self {
            Self {
                name,
                focused: false,
            }
        }
    }

    impl WithNavIndex for TestElement {
        fn nav_index(&self) -> Option<usize> {
            Some(0)
        }
    }

    impl Focusable<TestContext> for TestElement {
        fn is_focused(&self) -> bool {
            println!("Is '{}' focused? {}", self.name, self.focused);
            self.focused
        }

        fn set_focused(&mut self, focused: bool) {
            println!("Setting focus of '{}' to {}", self.name, focused);
            self.focused = focused;
        }
    }

    impl Element<TestContext> for TestElement {}

    struct TestParentElement {
        name: &'static str,
        children: Vec<Box<dyn Element<TestContext>>>,
        dragging_buttons: Vec<TestButton>,
    }

    impl TestParentElement {
        const fn new(name: &'static str) -> Self {
            Self {
                name,
                children: Vec::new(),
                dragging_buttons: Vec::new(),
            }
        }
    }

    impl WithNavIndex for TestParentElement {
        fn nav_index(&self) -> Option<usize> {
            Some(0)
        }
    }

    impl ParentElementFocusableImpl<TestContext> for TestParentElement {}

    impl Focusable<TestContext> for TestParentElement {
        fn is_focused(&self) -> bool {
            println!(
                "Is parent '{}' focused? {}",
                self.name,
                <Self as ParentElementFocusableImpl<_>>::is_focused(self)
            );
            <Self as ParentElementFocusableImpl<_>>::is_focused(self)
        }

        fn set_focused(&mut self, focused: bool) {
            println!("Setting focus of parent '{}' to {}", self.name, focused);
            <Self as ParentElementFocusableImpl<_>>::set_focused(self, focused);
        }
    }

    impl ParentElementExt<TestContext> for TestParentElement {}

    impl ParentElement<TestContext> for TestParentElement {
        fn children(&self) -> &[Box<dyn Element<TestContext>>] {
            &self.children
        }

        fn children_mut(&mut self) -> &mut [Box<dyn Element<TestContext>>] {
            &mut self.children
        }

        fn dragging_buttons(&self) -> &[TestButton] {
            &self.dragging_buttons
        }

        fn dragging_buttons_mut(&mut self) -> &mut Vec<TestButton> {
            &mut self.dragging_buttons
        }
    }

    #[test]
    fn test_instances() {
        let element = TestElement::new("leaf");
        let mut parent_element = TestParentElement::new("parent");
        parent_element.children.push(Box::new(element));

        let leaf = GuiNavigationPath::leaf(element);
        let mut node = GuiNavigationPath::node(parent_element, leaf);

        match node {
            GuiNavigationPath::Node {
                ref mut element,
                ref mut child,
            } => {
                assert!(!element.as_ref().is_focused());
                assert!(!child.as_ref().is_focused());

                println!("Focusing element...");
                element.focus();
                assert!(element.as_ref().is_focused());
                assert!(child.as_ref().is_focused());

                println!("Setting element unfocused...");
                element.set_focused(false);
                assert!(!element.as_ref().is_focused());
                assert!(!child.as_ref().is_focused());

                println!("Focusing child...");
                child.focus();
                assert!(child.as_ref().is_focused());
                assert!(element.as_ref().is_focused());

                println!("Setting child unfocused...");
                child.set_focused(false);
                assert!(!child.as_ref().is_focused());
                assert!(!element.as_ref().is_focused());
            }
            GuiNavigationPath::Leaf(_) => { /* unreachable */ }
        }
    }

    // #[test]
    // fn test_navigation() {
    //     let element = TestElement::new("leaf");
    //     let parent_element = TestParentElement::new("parent");

    //     let leaf = leaf(element);
    //     let parents: Vec<Box<dyn ParentElement<TestContext>>> = vec![
    //         Box::new(TestParentElement::new("parent1")),
    //         Box::new(TestParentElement::new("parent2")),
    //     ];

    //     let path = path(element, parents);
    // }
}
