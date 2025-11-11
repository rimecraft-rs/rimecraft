#![allow(missing_docs)]
#![cfg(test)]

use key_bind::{Key, KeyBind, KeyBindMode, KeyBindOp as _};
use keyboard::KeyState;
use std::cell::RefCell;
use std::rc::Rc;
use test_global::TestContext;
use test_global::integration::keyboard::TestKey;
use test_global::integration::mouse::TestButton;

#[test]
fn key_bind_handle_release() {
    let mode = Rc::new(RefCell::new(KeyBindMode::Hold));
    let mode_for_getter = mode.clone();

    let mut key_bind = KeyBind::<TestContext>::new(
        Box::new(move || *mode_for_getter.borrow()),
        Key::KeyboardKey(TestKey::Num0),
        (),
    );

    {
        // Presses the key.
        let handle = key_bind.press();
        assert_eq!(handle.state(), KeyState::Pressed);

        // Releases the key.
        drop(handle);
        assert_eq!(key_bind.state(), KeyState::Idle);

        *mode.borrow_mut() = KeyBindMode::Toggle;

        // Presses the key.
        let handle = key_bind.press();
        assert_eq!(handle.state(), KeyState::Pressed);

        // The mode is still frozen in the handle.
        *mode.borrow_mut() = KeyBindMode::Hold;

        // Releases the key.
        drop(handle);
        assert_eq!(key_bind.state(), KeyState::Idle);

        // Presses the key.
        let mut handle = key_bind.press();
        assert_eq!(handle.state(), KeyState::Pressed);

        // Forcefully resets to idle.
        handle.reset_to_idle();
        assert_eq!(handle.state(), KeyState::Idle);
        drop(handle);
    }
}

#[test]
fn key_bind_default_key() {
    let key_bind = KeyBind::<TestContext, ()> {
        mode_getter: Box::new(|| KeyBindMode::Hold),
        default_key: Key::KeyboardKey(TestKey::A),
        bound_key: None,
        state: KeyState::Idle,
        press_count: 0,
        ext: (),
    };

    assert_eq!(key_bind.default_key(), &Key::KeyboardKey(TestKey::A));
    assert!(key_bind.bound_key().is_none());
}

#[test]
fn key_bind_binding() {
    let mut key_bind = KeyBind::<TestContext, ()> {
        mode_getter: Box::new(|| KeyBindMode::Hold),
        default_key: Key::KeyboardKey(TestKey::A),
        bound_key: None,
        state: KeyState::Idle,
        press_count: 0,
        ext: (),
    };

    assert_eq!(key_bind.bound_key(), None);
    assert_eq!(key_bind.effective_key(), &Key::KeyboardKey(TestKey::A));

    key_bind.bind(Key::KeyboardKey(TestKey::B));

    assert_eq!(key_bind.bound_key(), Some(&Key::KeyboardKey(TestKey::B)));
    assert_eq!(key_bind.effective_key(), &Key::KeyboardKey(TestKey::B));

    key_bind.unbind();

    assert_eq!(key_bind.bound_key(), None);
    assert_eq!(key_bind.effective_key(), &Key::KeyboardKey(TestKey::A));

    // Also using traits:

    key_bind.bind(Key::KeyboardKey(keyboard::key::KeyNum::NUM_0));

    assert_eq!(key_bind.bound_key(), Some(&Key::KeyboardKey(TestKey::Num0)));
    assert_eq!(key_bind.effective_key(), &Key::KeyboardKey(TestKey::Num0));

    key_bind.bind(Key::MouseButton(mouse::button::MouseButton::BUTTON_PRIMARY));

    assert_eq!(
        key_bind.bound_key(),
        Some(&Key::MouseButton(TestButton::Left))
    );
}
