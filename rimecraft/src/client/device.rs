use std::time::Instant;

use glium::glutin::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton},
    window::WindowId,
};

#[derive(Default)]
pub struct Mouse {
    left_button_clicked: bool,
    middle_button_clicked: bool,
    right_button_clicked: bool,
    position: PhysicalPosition<f64>,
    cursor_locked: bool,
    touchscreen_handle: f64,
    active_button: Option<MouseButton>,
    instant: Option<Instant>,
}

impl Mouse {
    fn on_mouse_button(window: WindowId, button: MouseButton, state: ElementState) {
        let bl = matches!(state, ElementState::Pressed);
        let binding = super::INSTANCE.read().unwrap();
        let client = binding.as_ref().unwrap();
        if window != client.get_window().id() {
            return;
        }

        let mut mouse = client.mouse.write().unwrap();
        if bl {
            if client.options.read().unwrap().container.touchscreen && {
                let mut wm = mouse.touchscreen_handle;
                wm += 1_f64;
                wm
            } > 0_f64
            {
                return;
            }
            mouse.active_button = Some(button);
            mouse.instant = Some(Instant::now())
        } else if mouse.active_button.is_some() {
            if client.options.read().unwrap().container.touchscreen && {
                let mut wm = mouse.touchscreen_handle;
                wm -= 1_f64;
                wm
            } > 0_f64
            {
                return;
            }
            mouse.active_button = None
        }
        //TODO: let mut bl2 = false;
    }
}
