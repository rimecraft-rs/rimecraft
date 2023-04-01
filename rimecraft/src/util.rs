pub mod json_helper {
    use serde_json::{Map, Value};
    use std::io::{Error, ErrorKind};

    pub fn get_str<'a>(object: &'a Map<String, Value>, element: &'a str) -> Result<&'a str, Error> {
        if let Some(value) = object.get(element) {
            if let Some(st) = value.as_str() {
                return Ok(st);
            }
        }
        Err(Error::new(
            ErrorKind::Other,
            format!("Missing {element}, expected to find a String"),
        ))
    }

    pub fn get_bool<'a>(object: &'a Map<String, Value>, element: &'a str) -> Result<bool, Error> {
        if let Some(value) = object.get(element) {
            if let Some(st) = value.as_bool() {
                return Ok(st);
            }
        }
        Err(Error::new(
            ErrorKind::Other,
            format!("Missing {element}, expected to find a Boolean"),
        ))
    }

    pub fn get_i64<'a>(object: &'a Map<String, Value>, element: &'a str) -> Result<i64, Error> {
        if let Some(value) = object.get(element) {
            if let Some(st) = value.as_i64() {
                return Ok(st);
            }
        }
        Err(Error::new(
            ErrorKind::Other,
            format!("Missing {element}, expected to find a Integer"),
        ))
    }

    pub fn get_object<'a>(
        object: &'a Map<String, Value>,
        element: &'a str,
    ) -> Result<&'a Map<String, Value>, Error> {
        if let Some(value) = object.get(element) {
            if let Some(st) = value.as_object() {
                return Ok(st);
            }
        }
        Err(Error::new(
            ErrorKind::Other,
            format!("Missing {element}, expected to find a JsonObject"),
        ))
    }
}

pub mod uuids {
    use md5::{Digest, Md5};
    use uuid::Uuid;

    pub fn get_offline_player_uuid(nickname: &str) -> Uuid {
        name_from(format!("OfflinePlayer:{nickname}").as_bytes())
    }

    pub fn name_from(obj: impl AsRef<[u8]>) -> Uuid {
        let mut hasher = Md5::new();
        hasher.update(obj);
        let mut hash: [u8; 16] = hasher.finalize().into();
        hash[6] &= 0x0f;
        hash[6] |= 0x30;
        hash[8] &= 0x3f;
        hash[8] |= 0x80;
        Uuid::from_bytes(hash)
    }
}
