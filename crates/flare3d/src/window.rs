use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::state::State;

pub struct Window<'w> {
    state: State<'w>,
}

impl<'w> Window<'w> {
    pub fn new() -> Window<'w> {
        let event_loop = EventLoop::new().unwrap();
        let winit_window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut f3d_window = Window {
            state: futures_executor::block_on(State::new(&winit_window)),
        };

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
                    Event::WindowEvent { window_id, event } if window_id == winit_window.id() => {
                        if !f3d_window.state.input(&event) {
                            match event {
                                WindowEvent::CloseRequested => target.exit(),
                                WindowEvent::Resized(physical_size) => {
                                    f3d_window.state.resize(physical_size);
                                }
                                WindowEvent::RedrawRequested => {
                                    f3d_window.state.update();
                                    match f3d_window.state.render() {
                                        Ok(_) => {}
                                        Err(wgpu::SurfaceError::Lost) => {
                                            f3d_window.state.resize(f3d_window.state.size)
                                        }
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

                        winit_window.request_redraw();
                    }
                    _ => (),
                }
            })
            .unwrap();

        f3d_window
    }
}
