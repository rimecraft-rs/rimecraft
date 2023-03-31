use std::str::FromStr;

use uuid::Uuid;

pub enum AccountType {
    Legacy,
}

impl AccountType {
    pub fn get_name(&self) -> &str {
        match self {
            AccountType::Legacy => "legacy",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "legacy" => Some(Self::Legacy),
            _ => None,
        }
    }
}

pub struct Session {
    username: String,
    uuid: String,
    access_token: String,
    account_type: AccountType,
}

impl Session {
    pub fn new(
        username: String,
        uuid: String,
        access_token: String,
        account_type: AccountType,
    ) -> Self {
        Self {
            username,
            uuid,
            access_token,
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

    pub fn get_uuid_optional(&self) -> Option<Uuid> {
        match Uuid::from_str(&self.uuid) {
            Ok(uuid) => Some(uuid),
            Err(_) => None,
        }
    }
}
