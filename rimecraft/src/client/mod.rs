pub mod blaze3d;
pub mod device;
pub mod gui;
pub mod main;
pub mod option;
pub mod render;
pub mod resource;
pub mod util;

use self::{args::RunArgs, device::Mouse, option::GameOptions, util::Session};
use crate::{
    consts,
    network::Proxy,
    util::event::{default_phase, Event},
};
use glium::glutin::{
    dpi::PhysicalSize,
    event,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use log::{debug, info};
use once_cell::sync::Lazy;
use std::{rc::Rc, sync::RwLock, thread};

pub static INSTANCE: Lazy<RwLock<Option<RimecraftClient>>> = Lazy::new(|| RwLock::new(None));

pub static WINDOW_EVENT: Lazy<RwLock<Event<Rc<event::Event<'_, ()>>, Option<ControlFlow>>>> =
    Lazy::new(|| {
        RwLock::new(Event::new(
            Box::new(|callbacks, event: Rc<event::Event<'_, ()>>| {
                for callback in callbacks {
                    if let Some(b) = callback(event.clone()) {
                        return Some(b);
                    }
                }
                None
            }),
            Box::new(|_| None),
            vec![default_phase()],
        ))
    });

#[cfg(not(target_pointer_width = "64"))]
pub const IS_64_BIT: bool = false;
#[cfg(target_pointer_width = "64")]
pub const IS_64_BIT: bool = true;

#[cfg(not(target_os = "macos"))]
pub const IS_MACOS: bool = false;
#[cfg(target_os = "macos")]
pub const IS_MACOS: bool = true;

pub struct RimecraftClient {
    run_dir: String,
    resource_pack_dir: String,
    game_version: String,
    version_type: String,
    network_proxy: Proxy,
    session: Session,
    window: Window,
    pub options: RwLock<GameOptions>,
    pub mouse: RwLock<Mouse>,
}

impl RimecraftClient {
    pub const GL_ERROR_DIALOGUE: &str = "Please make sure you have up-to-date drivers.";

    pub fn new(args: RunArgs) -> Self {
        let s = Self {
            options: RwLock::new(GameOptions::new(&args.directories.run_dir)),
            run_dir: args.directories.run_dir,
            resource_pack_dir: args.directories.resource_pack_dir,
            game_version: args.game.version,
            version_type: args.game.version_type,
            network_proxy: args.network.net_proxy,
            session: args.network.session,
            window: {
                let event_loop: EventLoopContainer = EventLoopContainer::new(EventLoop::new());
                let window = WindowBuilder::new()
                    .with_inner_size(PhysicalSize::new(
                        args.window_settings.width,
                        args.window_settings.height,
                    ))
                    .build(&event_loop.inner)
                    .unwrap();

                thread::spawn(move || {
                    let el = event_loop;
                    el.inner.run(|event, _, flow| {
                        flow.set_wait();
                        if let Some(st) = event.to_static() {
                            if let Some(flow_r) = WINDOW_EVENT.read().unwrap().invoke(Rc::new(st)) {
                                *flow = flow_r;
                            }
                        }
                    });
                });
                window
            },
            mouse: RwLock::new(Mouse::default()),
        };
        s.update_window_title();
        info!("Setting user: {} ", s.session.get_username());
        debug!("(Session ID is {})", s.session.get_session_id());
        s
    }

    pub fn update_window_title(&self) {
        self.window.set_title(&self.get_window_title())
    }

    fn get_window_title(&self) -> String {
        let mut string = "Rimecraft".to_string();
        string.push(' ');
        string.push_str(consts::GAME_VERSION.get_name());
        string
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn get_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}

struct EventLoopContainer {
    pub inner: EventLoop<()>,
}

impl EventLoopContainer {
    pub fn new(event_loop: EventLoop<()>) -> Self {
        Self { inner: event_loop }
    }
}

unsafe impl Send for EventLoopContainer {}

pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
}

impl WindowSettings {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub mod args {
    use super::{util::Session, WindowSettings};
    use crate::network::Proxy;

    pub struct RunArgs {
        pub network: Network,
        pub window_settings: WindowSettings,
        pub directories: Directions,
        pub game: Game,
    }

    impl RunArgs {
        pub fn new(
            network: Network,
            window_settings: WindowSettings,
            directories: Directions,
            game: Game,
        ) -> Self {
            Self {
                network,
                window_settings,
                directories,
                game,
            }
        }
    }

    pub struct Network {
        pub session: Session,
        pub net_proxy: Proxy,
    }

    impl Network {
        pub fn new(session: Session, proxy: Proxy) -> Self {
            Self {
                session,
                net_proxy: proxy,
            }
        }
    }

    pub struct Directions {
        pub run_dir: String,
        pub resource_pack_dir: String,
        pub assets_dir: String,
        pub asset_index: Option<String>,
    }

    impl Directions {
        pub fn new(
            run_dir: String,
            res_pack_dir: String,
            asset_dir: String,
            asset_index: Option<String>,
        ) -> Self {
            Self {
                run_dir,
                resource_pack_dir: res_pack_dir,
                assets_dir: asset_dir,
                asset_index,
            }
        }

        pub fn get_asset_dir(&self) -> String {
            if self.asset_index.is_none() {
                self.assets_dir.clone()
            } else {
                todo!()
            }
        }
    }

    pub struct Game {
        pub version: String,
        pub version_type: String,
    }

    impl Game {
        pub fn new(version: String, version_type: String) -> Self {
            Self {
                version,
                version_type,
            }
        }
    }
}
