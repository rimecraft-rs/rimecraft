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
