use flare3d::window::Window;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[allow(unused_variables)]
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let winit_window = WindowBuilder::new().build(&event_loop).unwrap();

	let window = Window::new(event_loop, &winit_window);
}
