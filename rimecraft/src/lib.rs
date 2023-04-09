pub mod bootstrap;
pub mod client;
pub mod network;
pub mod registry;
pub mod resource;
pub mod util;

pub mod consts {
    use once_cell::sync::Lazy;
    use tokio::sync::RwLock;

    use crate::version::{GameVersion, RimecraftVersion};

    pub const SNBT_TOO_OLD_THRESHOLD: i64 = 3318;

    pub static GAME_VERSION: Lazy<RwLock<Option<Box<dyn GameVersion + Send + Sync>>>> =
        Lazy::new(|| RwLock::new(None));

    pub fn get_protocol_version() -> i64 {
        762
    }

    pub async fn set_game_version(game_version: impl GameVersion + Send + Sync + 'static) {
        *GAME_VERSION.write().await = Some(Box::new(game_version));
    }

    pub async fn create_game_version() {
        if GAME_VERSION.read().await.is_none() {
            *GAME_VERSION.write().await = Some(Box::new(RimecraftVersion::create().unwrap()));
        }
    }
}

pub mod version {
    use std::{fs::File, io::Error, io::Read, str::FromStr};

    use crate::{resource::ResourceType, util::json_helper};
    use chrono::{NaiveDate, Utc};
    use log::warn;

    #[derive(Clone)]
    pub struct RimecraftVersion {
        id: String,
        name: String,
        stable: bool,
        save_version: SaveVersion,
        protocol_version: i64,
        resource_pack_version: i64,
        data_pack_version: i64,
        build_time: NaiveDate,
    }

    impl RimecraftVersion {
        pub fn create() -> Result<Self, Error> {
            if let Ok(mut file) = File::open("./version.json") {
                if let Some(v) = {
                    let json = serde_json::Value::from_str(&{
                        let mut s = String::new();
                        let _result = file.read_to_string(&mut s);
                        s
                    })?;
                    match json {
                        serde_json::Value::Object(o) => match Self::new_from_json(&o) {
                            Ok(s) => Some(s),
                            Err(_) => None,
                        },
                        _ => None,
                    }
                } {
                    return Ok(v);
                } else {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Game version information is corrupt",
                    ));
                }
            }

            warn!("Missing version information!");
            Ok(Self::default())
        }

        fn new_from_json(json: &serde_json::Map<String, serde_json::Value>) -> Result<Self, Error> {
            let id = json_helper::get_str(&json, "id")?;
            let name = json_helper::get_str(&json, "name")?;
            let stable = json_helper::get_bool(json, "stable")?;
            let save_version = SaveVersion::new(
                json_helper::get_i64(json, "world_version")?,
                json_helper::get_str(json, "series_id")
                    .unwrap_or(&SaveVersion::get_main_series())
                    .to_owned(),
            );
            let protocol_version = json_helper::get_i64(json, "resource")?;
            let json_object = json_helper::get_object(json, "pack_version")?;
            let resource_pack_version = json_helper::get_i64(json_object, "resource")?;
            let data_pack_version = json_helper::get_i64(json_object, "data")?;
            drop(resource_pack_version);

            let build_time;
            let build_time_raw: Result<chrono::DateTime<Utc>, _> =
                chrono::DateTime::from_str(json_helper::get_str(json, "build_time")?);
            if let Ok(b) = build_time_raw {
                build_time = b.date_naive();
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Build time format error",
                ));
            }

            Ok(Self {
                id: id.to_owned(),
                name: name.to_owned(),
                stable,
                save_version,
                protocol_version,
                resource_pack_version,
                data_pack_version,
                build_time,
            })
        }
    }

    impl Default for RimecraftVersion {
        fn default() -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string().replace("-", ""),
                name: String::from("1.19.4"),
                stable: true,
                save_version: SaveVersion::new(3337, String::from("main")),
                protocol_version: super::consts::get_protocol_version(),
                resource_pack_version: 13,
                data_pack_version: 12,
                build_time: NaiveDate::default(),
            }
        }
    }

    impl GameVersion for RimecraftVersion {
        fn get_save_version(&self) -> &SaveVersion {
            &self.save_version
        }

        fn get_id(&self) -> &str {
            &self.id
        }

        fn get_name(&self) -> &str {
            &self.name
        }

        fn get_protocol_version(&self) -> i64 {
            self.protocol_version
        }

        fn get_resource_version(&self, res: &ResourceType) -> i64 {
            match res {
                ResourceType::ClientResources => self.resource_pack_version,
                ResourceType::ServerData => self.data_pack_version,
            }
        }

        fn get_build_time(&self) -> NaiveDate {
            self.build_time
        }

        fn is_stable(&self) -> bool {
            self.stable
        }
    }

    pub trait GameVersion {
        fn get_save_version(&self) -> &SaveVersion;
        fn get_id(&self) -> &str;
        fn get_name(&self) -> &str;
        fn get_protocol_version(&self) -> i64;
        fn get_resource_version(&self, res: &ResourceType) -> i64;
        fn get_build_time(&self) -> NaiveDate;
        fn is_stable(&self) -> bool;
    }

    #[derive(Clone)]
    pub struct SaveVersion {
        id: i64,
        series: String,
    }

    impl SaveVersion {
        pub fn get_main_series() -> String {
            String::from("main")
        }

        pub fn new_default(id: i64) -> Self {
            Self {
                id,
                series: Self::get_main_series(),
            }
        }

        pub fn new(id: i64, series: String) -> Self {
            Self { id, series }
        }

        pub fn is_not_main_series(&self) -> bool {
            !self.series.eq(&Self::get_main_series())
        }

        pub fn get_series(&self) -> &str {
            &self.series
        }

        pub fn get_id(&self) -> i64 {
            self.id
        }

        pub fn is_available_to(&self, other: &SaveVersion) -> bool {
            self.get_series().eq(other.get_series())
        }
    }
}
