//! Essential tests for rimecraft-client-ui that require a context.

#![allow(missing_docs)]
#![cfg(test)]

use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use arena::Arena;
use local_cx::LocalContext;
use test_global::{
    TestContext,
    integration::{arena::TestArena, mouse::TestButton, ui::ArenaLocalContext},
};
use ui::{
    Element, Focusable, ParentElement, ParentElementExt, ParentElementFocusableImpl, ProvideUiTy,
    nav::{WithNavIndex, gui::GuiNavigationPath},
};

#[derive(Debug)]
enum TestValue {
    Number(i32),
}

#[derive(Debug)]
struct TestElement<'a> {
    name: &'static str,
    focused: bool,
    value: Option<TestValue>,
    local_cx: &'a ArenaLocalContext<'a>,
}

impl<'a> TestElement<'a> {
    fn new(name: &'static str, arena: &'a ArenaLocalContext<'a>) -> Self {
        Self {
            name,
            focused: false,
            value: None,
            local_cx: arena,
        }
    }
}

impl WithNavIndex for TestElement<'_> {
    fn nav_index(&self) -> Option<usize> {
        Some(0)
    }
}

impl Focusable<TestContext> for TestElement<'_> {
    fn is_focused(&self) -> bool {
        println!("Is '{:?}' focused? {}", self, self.focused);
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        println!("Setting focus of '{:?}' to {}", self, focused);
        self.focused = focused
    }
}

impl<'a> Element<'a, TestContext> for TestElement<'a> {}

#[derive(Debug)]
struct TestParentElement<'a> {
    name: &'static str,
    handles: Vec<<<TestContext as ProvideUiTy<'a>>::Arena as Arena>::Handle>,
    dragging_buttons: Vec<TestButton>,
    arena: &'a ArenaLocalContext<'a>,
}

impl<'a> TestParentElement<'a> {
    fn new(name: &'static str, arena: &'a ArenaLocalContext<'a>) -> Self {
        Self {
            name,
            handles: Vec::new(),
            dragging_buttons: Vec::new(),
            arena,
        }
    }
}

impl WithNavIndex for TestParentElement<'_> {
    fn nav_index(&self) -> Option<usize> {
        Some(0)
    }
}

impl<'a> ParentElementFocusableImpl<'a, TestContext> for TestParentElement<'a> {}

impl Focusable<TestContext> for TestParentElement<'_> {
    fn is_focused(&self) -> bool {
        println!(
            "Is parent '{:?}' focused? {}",
            self,
            <Self as ParentElementFocusableImpl<_>>::is_focused(self)
        );
        <Self as ParentElementFocusableImpl<_>>::is_focused(self)
    }

    fn set_focused(&mut self, focused: bool) {
        println!("Setting focus of parent '{:?}' to {}", self, focused);
        <Self as ParentElementFocusableImpl<_>>::set_focused(self, focused);
    }
}

impl<'a> ParentElementExt<'a, TestContext> for TestParentElement<'a> {}

impl<'a> ParentElement<'a, TestContext> for TestParentElement<'a> {
    fn local_cx(&self) -> <TestContext as ProvideUiTy<'a>>::ArenaLocalContext {
        self.arena
    }

    fn handles(&self) -> &[<<TestContext as ProvideUiTy<'a>>::Arena as Arena>::Handle] {
        &self.handles
    }

    fn handles_mut(
        &mut self,
    ) -> &mut Vec<<<TestContext as ProvideUiTy<'a>>::Arena as Arena>::Handle> {
        &mut self.handles
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
    let arena = RwLock::new(TestArena::new());
    let arena_local_cx = ArenaLocalContext::new(Arc::new(arena.read().unwrap()));

    let element = TestElement::new("leaf", &arena_local_cx);
    let element_handle = arena.insert(RefCell::new(Box::new(element)));

    let mut parent_element = TestParentElement::new("parent", &arena_local_cx);
    let parent_element_handle = arena.insert(RefCell::new(Box::new(parent_element)));

    parent_element.handles.push(element_handle);

    let leaf = GuiNavigationPath::<'_, TestContext>::leaf(element_handle);
    let mut node = GuiNavigationPath::<'_, TestContext>::node(parent_element_handle, leaf);

    // match node {
    //     GuiNavigationPath::Node {
    //         ref mut element,
    //         ref mut child,
    //     } => {
    //         assert!(!element.is_focused());
    //         assert!(!child.is_focused());

    //         println!("Focusing element...");
    //         element.focus();
    //         assert!(element.is_focused());
    //         assert!(child.is_focused());

    //         println!("Setting element unfocused...");
    //         element.set_focused(false);
    //         assert!(!element.is_focused());
    //         assert!(!child.is_focused());

    //         println!("Focusing child...");
    //         child.focus();
    //         assert!(element.is_focused());
    //         assert!(child.is_focused());

    //         println!("Setting child unfocused...");
    //         child.set_focused(false);
    //         assert!(!element.is_focused());
    //         assert!(!child.is_focused());
    //     }
    //     GuiNavigationPath::Leaf(_) => { /* unreachable */ }
    // }
}

// #[test]
// fn test_navigation() {
//     let arena = Arc::new(TestArena::new());
//     let arena_local_cx = ArenaLocalContext::new(&arena);

//     let element = TestElement::new("leaf", &arena_local_cx);
//     let parent_element = TestParentElement::new("parent", &arena_local_cx);

//     let leaf = leaf(element);
//     let parents: Vec<Box<dyn ParentElement<TestContext>>> = vec![
//         Box::new(TestParentElement::new("parent1", &arena_local_cx)),
//         Box::new(TestParentElement::new("parent2", &arena_local_cx)),
//     ];

//     let path = path(element, parents);
// }
