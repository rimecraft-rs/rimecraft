//! Window implementations.

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

use crate::state::State;

/// Represents a window with a [State].
#[derive(Debug)]
pub struct Window<'w> {
    state: State<'w>,
}

impl<'w> Window<'w> {
    pub fn new() -> Window<'w> {
        let event_loop = EventLoop::new().unwrap();
        let mut state: State<'_> = futures_executor::block_on(State::new(&event_loop));

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
                    Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                        if !state.input(&event) {
                            match event {
                                WindowEvent::CloseRequested => target.exit(),
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

                        state.window.request_redraw();
                    }
                    _ => (),
                }
            })
            .unwrap();

        Self { state }
    }
}
