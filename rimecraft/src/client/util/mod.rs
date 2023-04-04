pub mod render;

use std::str::FromStr;
use uuid::Uuid;

pub enum AccountType {
    Legacy,
    MSA,
    MOJANG,
}

impl AccountType {
    pub fn get_name(&self) -> &str {
        match self {
            AccountType::Legacy => "legacy",
            AccountType::MSA => "mojang",
            AccountType::MOJANG => "msa",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "legacy" => Some(Self::Legacy),
            "msa" => Some(Self::MSA),
            "mojang" => Some(Self::MOJANG),
            _ => None,
        }
    }
}

impl Default for AccountType {
    fn default() -> Self {
        Self::Legacy
    }
}

pub struct Session {
    username: String,
    uuid: String,
    access_token: String,
    xuid: Option<String>,
    client_id: Option<String>,
    account_type: AccountType,
}

impl Session {
    pub fn new(
        username: String,
        uuid: String,
        access_token: String,
        xuid: Option<String>,
        client_id: Option<String>,
        account_type: AccountType,
    ) -> Self {
        Self {
            username,
            uuid,
            access_token,
            xuid,
            client_id,
            account_type,
        }
    }

    pub fn get_session_id(&self) -> String {
        format!("token:{}:{}", self.access_token, self.uuid)
    }

    pub fn get_uuid(&self) -> &str {
        &self.access_token
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_access_token(&self) -> &str {
        &self.access_token
    }

    pub fn get_client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    pub fn get_xuid(&self) -> Option<&str> {
        self.xuid.as_deref()
    }

    pub fn get_uuid_or_none(&self) -> Option<Uuid> {
        match Uuid::from_str(&self.uuid) {
            Ok(uuid) => Some(uuid),
            Err(_) => None,
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
