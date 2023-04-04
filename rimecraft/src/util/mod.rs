pub mod json_helper;
pub mod system_details;
pub mod uuids;

use std::process::Command;
use url::Url;

pub fn into_option<T, U>(result: Result<T, U>) -> Option<T> {
    match result {
        Ok(obj) => Some(obj),
        Err(_) => None,
    }
}

pub fn get_operation_system() -> OperationSystem {
    match std::env::consts::OS {
        "linux" => OperationSystem::GnuLinux,
        "macos" => OperationSystem::MacOS,
        "windows" => OperationSystem::Windows,
        "solaris" => OperationSystem::Solaris,
        _ => OperationSystem::Unknown,
    }
}

pub enum OperationSystem {
    GnuLinux,
    Solaris,
    Windows,
    MacOS,
    Unknown,
}

impl OperationSystem {
    pub fn get_name(&self) -> String {
        match &self {
            OperationSystem::GnuLinux => String::from("linux"),
            OperationSystem::Solaris => String::from("solaris"),
            OperationSystem::Windows => String::from("windows"),
            OperationSystem::MacOS => String::from("mac"),
            OperationSystem::Unknown => String::from("unknown"),
        }
    }

    pub fn open_url(&self, url: Url) {
        let _output = self.get_url_open_command(&url).output();
    }

    fn get_url_open_command(&self, url: &Url) -> Command {
        match &self {
            OperationSystem::Windows => {
                let mut command = Command::new("rundll32");
                command.arg("url.dll,FileProtocolHandler");
                command.arg(url.as_str());
                command
            }
            OperationSystem::MacOS => {
                let mut command = Command::new("open");
                command.arg(url.as_str());
                command
            }
            _ => {
                let mut string = url.as_str().to_owned();
                if let Some(s) = string.strip_prefix("file:") {
                    string = format!("file://{s}");
                }
                let mut command = Command::new("xdg-open");
                command.arg(string);
                command
            }
        }
    }
}
