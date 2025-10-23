//! Essential tests for rimecraft-client-ui that require a context.

#![allow(missing_docs)]
#![cfg(test)]

use std::sync::RwLock;

use test_global::{TestContext, integration::mouse::TestButton};
use ui::{
    Element, ImmutablyFocusable, ParentElement, ParentElementExt, ParentElementFocusableImpl,
    nav::{WithNavIndex, gui::GuiNavigationPath},
};

#[derive(Debug)]
struct TestElement {
    name: &'static str,
    focused: RwLock<bool>,
}

impl TestElement {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            focused: RwLock::new(false),
        }
    }
}

impl WithNavIndex for TestElement {
    fn nav_index(&self) -> Option<usize> {
        Some(0)
    }
}

impl ImmutablyFocusable<TestContext> for TestElement {
    fn is_focused(&self) -> bool {
        let focused = self.focused.read().unwrap();
        println!("Is '{}' focused? {}", self.name, *focused);
        *focused
    }

    fn set_focused(&self, focused: bool) {
        println!("Setting focus of '{}' to {}", self.name, focused);
        let mut guard = self.focused.write().unwrap();
        *guard = focused;
    }
}

impl Element<TestContext> for TestElement {}

struct TestParentElement<'a> {
    name: &'static str,
    children: Vec<&'a mut dyn Element<TestContext>>,
    dragging_buttons: Vec<TestButton>,
}

impl TestParentElement<'_> {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            children: Vec::new(),
            dragging_buttons: Vec::new(),
        }
    }
}

impl WithNavIndex for TestParentElement<'_> {
    fn nav_index(&self) -> Option<usize> {
        Some(0)
    }
}

impl ParentElementFocusableImpl<TestContext> for TestParentElement<'_> {}

impl ImmutablyFocusable<TestContext> for TestParentElement<'_> {
    fn is_focused(&self) -> bool {
        println!(
            "Is parent '{}' focused? {}",
            self.name,
            <Self as ParentElementFocusableImpl<_>>::is_focused(self)
        );
        <Self as ParentElementFocusableImpl<_>>::is_focused(self)
    }

    fn set_focused(&self, focused: bool) {
        println!("Setting focus of parent '{}' to {}", self.name, focused);
        <Self as ParentElementFocusableImpl<_>>::set_focused(self, focused);
    }
}

impl ParentElementExt<TestContext> for TestParentElement<'_> {}

impl ParentElement<TestContext> for TestParentElement<'_> {
    fn children(&self) -> &[&dyn Element<TestContext>] {}

    fn children_mut(&mut self) -> &mut [&mut dyn Element<TestContext>] {}

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

    let leaf = GuiNavigationPath::leaf(&element);
    let mut node = GuiNavigationPath::node(&parent_element, leaf);

    match node {
        GuiNavigationPath::Node {
            ref mut element,
            ref mut child,
        } => {
            assert!(!element.is_focused());
            assert!(!child.is_focused());

            println!("Focusing element...");
            element.focus();
            assert!(element.is_focused());
            assert!(child.is_focused());

            println!("Setting element unfocused...");
            element.set_focused(false);
            assert!(!element.is_focused());
            assert!(!child.is_focused());

            println!("Focusing child...");
            child.focus();
            assert!(element.is_focused());
            assert!(child.is_focused());

            println!("Setting child unfocused...");
            child.set_focused(false);
            assert!(!element.is_focused());
            assert!(!child.is_focused());
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
