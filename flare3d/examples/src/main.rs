use flare3d::state::State;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = futures::executor::block_on(State::new(&window));

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        } => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        })
        .unwrap();
}
