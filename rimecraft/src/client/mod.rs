pub mod blaze3d;
pub mod main;
pub mod resource;
pub mod util;

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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub fn new(namespace: String, path: String) -> Option<Self> {
        if Self::is_namespace_valid(&namespace) && Self::is_path_valid(&path) {
            Some(Self::new_unchecked(namespace, path))
        } else {
            None
        }
    }

    pub fn parse(id: String) -> Option<Self> {
        Self::split_on(id, ':')
    }

    pub fn split_on(id: String, delimiter: char) -> Option<Self> {
        let arr = id.split_once(delimiter)?;
        Self::new(arr.0.to_string(), arr.1.to_string())
    }

    fn new_unchecked(namespace: String, path: String) -> Self {
        Self { namespace, path }
    }

    fn is_namespace_valid(namespace: &str) -> bool {
        for c in namespace.chars() {
            if !(c == '_' || c == '-' || c >= 'a' || c <= 'z' || c >= '0' || c <= '9' || c == '.') {
                return false;
            }
        }
        true
    }

    fn is_path_valid(path: &str) -> bool {
        for c in path.chars() {
            if !(c == '_'
                || c == '-'
                || c >= 'a'
                || c <= 'z'
                || c >= '0'
                || c <= '9'
                || c == '.'
                || c == '/')
            {
                return false;
            }
        }
        true
    }

    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}
