pub mod bootstrap;
pub mod client;
pub mod item;
pub mod nbt;
pub mod network;
pub mod registry;
pub mod resource;
#[cfg(test)]
mod tests;
pub mod transfer;
pub mod util;

mod error {
    use std::{error, fmt::Display, result};

    pub type Result<T> = result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error {
        Runtime(String),
        Encoder(String),
        Decoder(String),
        IllegalState(String),
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::Runtime(value) => f.write_str(value)?,
                Error::Decoder(value) => f.write_str(value)?,
                Error::Encoder(value) => f.write_str(value)?,
                Error::IllegalState(value) => f.write_str(value)?,
            }
            Ok(())
        }
    }

    impl error::Error for Error {}
}

pub mod consts {
    use crate::version::RimecraftVersion;

    use once_cell::sync::Lazy;

    pub static GAME_VERSION: Lazy<RimecraftVersion> =
        Lazy::new(|| RimecraftVersion::create().unwrap());

    pub fn get_protocol_version() -> u32 {
        1073741957
    }
}

pub mod version {
    use crate::{resource::ResourceType, util::json_helper};

    use chrono::{NaiveDate, Utc};
    use log::warn;
    use std::{fs::File, io::Error, io::Read, str::FromStr};

    #[derive(Clone)]
    pub struct RimecraftVersion {
        id: String,
        name: String,
        stable: bool,
        save_version: SaveVersion,
        protocol_version: u32,
        resource_pack_version: u32,
        data_pack_version: u32,
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
            let id = json_helper::get_str(json, "id")?;
            let name = json_helper::get_str(json, "name")?;
            let stable = json_helper::get_bool(json, "stable")?;

            let save_version = SaveVersion::new(
                json_helper::get_i64(json, "world_version")? as i32,
                json_helper::get_str(json, "series_id")
                    .unwrap_or(SaveVersion::MAIN_SERIES)
                    .to_owned(),
            );

            let protocol_version = json_helper::get_i64(json, "resource")?;
            let json_object = json_helper::get_object(json, "pack_version")?;
            let resource_pack_version = json_helper::get_i64(json_object, "resource")?;
            let data_pack_version = json_helper::get_i64(json_object, "data")?;

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
                protocol_version: protocol_version as u32,
                resource_pack_version: resource_pack_version as u32,
                data_pack_version: data_pack_version as u32,
                build_time,
            })
        }

        /// The save version information for this game version
        pub fn get_save_version(&self) -> &SaveVersion {
            &self.save_version
        }

        pub fn get_id(&self) -> &str {
            &self.id
        }

        pub fn get_name(&self) -> &str {
            &self.name
        }

        pub fn get_protocol_version(&self) -> u32 {
            self.protocol_version
        }

        pub fn get_resource_version(&self, res: &ResourceType) -> u32 {
            match res {
                ResourceType::ClientResources => self.resource_pack_version,
                ResourceType::ServerData => self.data_pack_version,
            }
        }

        pub fn get_build_time(&self) -> NaiveDate {
            self.build_time
        }

        pub fn is_stable(&self) -> bool {
            self.stable
        }
    }

    impl Default for RimecraftVersion {
        fn default() -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string().replace('-', ""),
                name: String::from("1.20"),
                stable: true,
                save_version: SaveVersion::new(3463, String::from("main")),
                protocol_version: super::consts::get_protocol_version(),
                resource_pack_version: 15,
                data_pack_version: 15,
                build_time: NaiveDate::default(),
            }
        }
    }

    /// The version components of Rimecraft that is used for identification in save games.
    #[derive(Clone)]
    pub struct SaveVersion {
        id: i32,
        series: String,
    }

    impl SaveVersion {
        /// The default series of a version, `main`, if a series is not specified.
        pub const MAIN_SERIES: &str = "main";

        pub fn new_default(id: i32) -> Self {
            Self {
                id,
                series: Self::MAIN_SERIES.to_string(),
            }
        }

        pub fn new(id: i32, series: String) -> Self {
            Self { id, series }
        }

        pub fn is_not_main_series(&self) -> bool {
            !self.series.eq(Self::MAIN_SERIES)
        }

        /// The series of this version
        ///
        /// This is stored in the `series` field within `level.dat`.
        pub fn series(&self) -> &str {
            &self.series
        }

        /// The integer data version of this save version
        pub fn id(&self) -> i32 {
            self.id
        }

        /// Whether this save version can be loaded by the `other` version
        pub fn is_available_to(&self, other: &SaveVersion) -> bool {
            self.series().eq(other.series())
        }
    }
}

pub use error::*;
