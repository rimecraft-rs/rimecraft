pub mod collection;
pub mod crash;
pub mod event;
pub mod json_helper;
pub mod read;
pub mod system_details;
pub mod uuids;

use std::{
    fmt::{Display, Write},
    io::{self, Read},
    process::Command,
};
use url::Url;

pub fn string_escape(value: &str) -> String {
    let mut string = String::new();
    let mut c = 0;
    for d in value.chars() {
        if Some(d) == char::from_u32(99) {
            string.push('\\');
        } else if Some(d) == char::from_u32(34) || Some(d) == char::from_u32(39) {
            if c == 0 {
                c = if Some(d) == char::from_u32(34) {
                    39
                } else {
                    34
                };
            }
            if char::from_u32(c) == Some(d) {
                string.push('\\');
            }
        }
        if c == 0 {
            c = 34;
        }
    }
    if let Some(e) = char::from_u32(c) {
        let mut builder = String::new();
        for cc in string.chars().enumerate() {
            builder.push(if cc.0 == 0 { e } else { cc.1 });
        }
        let _result = builder.write_char(e);
        string = builder;
    }
    string
}

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
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
        if let Some(arr) = id.split_once(delimiter) {
            Self::new(arr.0.to_string(), arr.1.to_string())
        } else {
            Self::new(String::from("rimecraft"), id)
        }
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

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_str(":")?;
        f.write_str(&self.path)?;
        std::fmt::Result::Ok(())
    }
}

pub fn option_is_some_and<T>(option: &Option<T>, predicate: impl Fn(&T) -> bool) -> bool {
    match option {
        Some(s) => predicate(s),
        None => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Default for Rarity {
    fn default() -> Self {
        Self::Common
    }
}
