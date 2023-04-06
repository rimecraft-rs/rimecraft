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
