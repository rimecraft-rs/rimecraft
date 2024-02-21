use flare3d::state::State;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = futures_executor::block_on(State::new(&window));

    #[cfg(target_os = "macos")]
    let (_dl, semaphore) = {
        use std::sync::{Arc, Condvar, Mutex};

        let pair = Arc::new((Mutex::new(false), Condvar::new()));

        let pair2 = Arc::clone(&pair);
        let mut dl = display_link::DisplayLink::new(move |_ts| {
            let (lock, cvar) = &*pair2;
            let mut do_redraw = lock.lock().unwrap();
            *do_redraw = true;

            cvar.notify_one();
        })
        .unwrap();

        dl.resume().unwrap();

        (dl, pair)
    };

    event_loop
        .run(|event, target| {
            target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    if !state.input(&event) {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => target.exit(),
                            WindowEvent::Resized(physical_size) => {
                                state.resize(physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                if state.do_render() {
                                    state.update();
                                    match state.render() {
                                        Ok(_) => {}
                                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                        Err(e) => eprintln!("{:?}", e),
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                Event::AboutToWait => {
                    #[cfg(target_os = "macos")]
                    {
                        let (lock, cvar) = &*semaphore;
                        let mut do_redraw = lock.lock().unwrap();

                        while !*do_redraw {
                            do_redraw = cvar.wait(do_redraw).unwrap();
                        }

                        *do_redraw = false;
                    }

                    window.request_redraw();
                }
                _ => (),
            }
        })
        .unwrap();
}
