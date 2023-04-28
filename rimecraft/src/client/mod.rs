use self::{args::RunArgs, util::Session};
use crate::{consts, network::Proxy, version::GameVersion};
use glium::glutin::{dpi::PhysicalSize, window::WindowBuilder};
use log::info;
use once_cell::{sync, unsync};
use std::sync::RwLock;

pub mod blaze3d;
pub mod main;
pub mod resource;
pub mod util;

pub static INSTANCE: sync::Lazy<RwLock<Option<RimecraftClientSynced>>> =
    sync::Lazy::new(|| RwLock::new(None));
pub static mut INSTANCE_UNSAFE: unsync::Lazy<Option<RimecraftClientUnsynced>> =
    unsync::Lazy::new(|| None);

pub struct RimecraftClientSynced {
    run_dir: String,
    resource_pack_dir: String,
    game_version: String,
    version_type: String,
    netowk_proxy: Proxy,
    session: Session,
}

pub struct RimecraftClientUnsynced {
    window: util::Window,
}

impl RimecraftClientSynced {
    pub fn new(args: RunArgs) -> Self {
        let s = Self {
            run_dir: args.directories.run_dir,
            resource_pack_dir: args.directories.resource_pack_dir,
            game_version: args.game.version,
            version_type: args.game.version_type,
            netowk_proxy: args.network.net_proxy,
            session: args.network.session,
        };
        info!("Setting user: {}", s.session.get_username());
        info!("(Session ID is {})", s.session.get_session_id());
        s
    }
}

impl RimecraftClientUnsynced {
    pub const GL_ERROR_DIALOGUE: &str = "Please make sure you have up-to-date drivers.";

    pub fn new(args: &RunArgs) -> Self {
        let client = Self {
            window: {
                self::util::Window::new(WindowBuilder::new().with_inner_size(PhysicalSize::new(
                    args.window_settings.width,
                    args.window_settings.height,
                )))
            },
        };
        client.update_window_title();
        client
    }

    pub fn update_window_title(&self) {
        self.window.get_window().set_title(&self.get_window_title())
    }

    fn get_window_title(&self) -> String {
        let mut string = "Rimecraft".to_string();
        string.push(' ');
        string.push_str(consts::GAME_VERSION.get_name());
        string
    }
}

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
