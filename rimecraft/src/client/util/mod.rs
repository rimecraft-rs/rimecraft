pub mod render;

use glium::glutin::{
    event_loop::EventLoop,
    monitor::MonitorHandle,
    window::{self, WindowBuilder},
};
use std::str::FromStr;
use uuid::Uuid;

pub struct Window {
    inner: Option<window::Window>,
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new(builder: WindowBuilder) -> Self {
        let mut s = Self {
            inner: None,
            event_loop: EventLoop::new(),
        };
        s.inner = Some(builder.build(&s.event_loop).unwrap());
        s
    }

    pub fn get_window(&self) -> &window::Window {
        self.inner.as_ref().unwrap()
    }

    pub fn get_window_mut(&mut self) -> &mut window::Window {
        self.inner.as_mut().unwrap()
    }

    pub fn get_event_loop(&self) -> &EventLoop<()> {
        &self.event_loop
    }

    pub fn get_evnt_loop_mut(&mut self) -> &mut EventLoop<()> {
        &mut self.event_loop
    }

    pub fn monitor_handler(&self) -> Option<MonitorHandle> {
        self.get_window().current_monitor()
    }
}

pub enum AccountType {
    Legacy,
    MSA,
}

impl AccountType {
    pub fn get_name(&self) -> &str {
        match self {
            AccountType::Legacy => "legacy",
            AccountType::MSA => "msa",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "legacy" => Some(Self::Legacy),
            "msa" => Some(Self::MSA),
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
