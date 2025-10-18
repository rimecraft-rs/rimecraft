use rimecraft_global_cx::GlobalContext;
use rimecraft_keyboard::key::KeyNum;
use rimecraft_mouse::button::MouseButton;
use std::cell::RefCell;
use std::rc::Rc;

use crate::*;

#[test]
fn test_key_bind_handle_release() {
    enum Key {
        Num0,
        Num1,
        Num2,
        Num3,
        Num4,
        Num5,
        Num6,
        Num7,
        Num8,
        Num9,
        ButtonLeft,
        ButtonRight,
        ButtonMiddle,
    }

    impl KeyNum for Key {
        const NUM_0: Self = Self::Num0;
        const NUM_1: Self = Self::Num1;
        const NUM_2: Self = Self::Num2;
        const NUM_3: Self = Self::Num3;
        const NUM_4: Self = Self::Num4;
        const NUM_5: Self = Self::Num5;
        const NUM_6: Self = Self::Num6;
        const NUM_7: Self = Self::Num7;
        const NUM_8: Self = Self::Num8;
        const NUM_9: Self = Self::Num9;
    }

    impl MouseButton for Key {
        const BUTTON_PRIMARY: Self = Self::ButtonLeft;
        const BUTTON_SECONDARY: Self = Self::ButtonRight;
        const BUTTON_SCROLL_WHEEL: Self = Self::ButtonMiddle;
    }

    struct TestCx;

    unsafe impl GlobalContext for TestCx {}

    impl ProvideKeyboardTy for TestCx {
        type Key = Key;
    }

    impl ProvideMouseTy for TestCx {
        type Button = Key;
    }

    let mode = Rc::new(RefCell::new(KeyBindMode::Hold));
    let mode_for_getter = mode.clone();

    let mut key_bind = KeyBind::<TestCx, ()> {
        mode_getter: Box::new(move || *mode_for_getter.borrow()),
        default_key: crate::Key::KeyboardKey(Key::NUM_0),
        bound_key: None,
        state: KeyState::Idle,
        press_count: 0,
        ext: (),
    };

    {
        // Presses the key.
        let handle = key_bind.press();
        assert_eq!(handle.state, KeyState::Pressed);

        // Releases the key.
        drop(handle);
        assert_eq!(key_bind.state, KeyState::Idle);

        *mode.borrow_mut() = KeyBindMode::Toggle;

        // Presses the key.
        let handle = key_bind.press();
        assert_eq!(handle.state, KeyState::Pressed);

        // The mode is still frozen in the handle.
        *mode.borrow_mut() = KeyBindMode::Hold;

        // Releases the key.
        drop(handle);
        assert_eq!(key_bind.state, KeyState::Idle);

        // Presses the key.
        let mut handle = key_bind.press();
        assert_eq!(handle.state, KeyState::Pressed);

        // Forcefully resets to idle.
        handle.reset_to_idle();
        assert_eq!(handle.state, KeyState::Idle);
        drop(handle);
    }
}
