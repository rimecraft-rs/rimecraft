pub mod main;
pub mod resource;
pub mod util;

pub mod args {
    use super::util::Session;
    use crate::network::Proxy;

    pub struct RunArgs {}

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
}
