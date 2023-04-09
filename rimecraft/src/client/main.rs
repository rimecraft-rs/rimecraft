use crate::{bootstrap, network::Proxy, util::uuids};
use chrono::Utc;
use clap::Parser;
use log::warn;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use super::{
    args::{self, RunArgs},
    util::{AccountType, Session},
};

#[derive(Debug, Parser)]
pub struct OptionSet {
    #[arg(long, default_value_t = format!("."))]
    pub game_dir: String,
    #[arg(long)]
    pub assets_dir: Option<String>,
    #[arg(long)]
    pub resource_pack_dir: Option<String>,
    #[arg(long)]
    pub proxy_host: Option<String>,
    #[arg(long, default_value_t = 8080)]
    pub proxy_port: u16,
    #[arg(long)]
    pub proxy_user: Option<String>,
    #[arg(long)]
    pub proxy_pass: Option<String>,
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub uuid: Option<String>,
    #[arg(long, default_value_t = format!(""))]
    pub xuid: String,
    #[arg(long, default_value_t = format!(""))]
    pub client_id: String,
    #[arg(long)]
    pub access_token: String,
    #[arg(long)]
    pub version: String,
    #[arg(long, default_value_t = 854)]
    pub width: u32,
    #[arg(long, default_value_t = 480)]
    pub height: u32,
    #[arg(long)]
    pub asset_index: Option<String>,
    #[arg(long, default_value_t = format!("legacy"))]
    pub user_type: String,
    #[arg(long, default_value_t = format!("release"))]
    pub version_type: String,
}

pub fn main(options: Option<OptionSet>) {
    let option_set = options.unwrap_or(OptionSet::parse());
    let mut proxy: Proxy = Proxy::NoProxy;
    if let Some(h) = &option_set.proxy_host {
        let addr: Result<Ipv4Addr, _> = h.parse();
        if let Ok(host) = addr {
            proxy = Proxy::SOCKS(
                SocketAddr::V4(SocketAddrV4::new(host, option_set.proxy_port)),
                if option_set.proxy_user.is_some() && option_set.proxy_pass.is_some() {
                    Some(crate::network::ProxyPassword(
                        option_set.proxy_user.unwrap().clone(),
                        option_set.proxy_pass.unwrap().clone(),
                    ))
                } else {
                    None
                },
            );
        }
    }
    let account_type = AccountType::by_name(&option_set.user_type);
    if account_type.is_none() {
        warn!("Unrecognized user type: {}", option_set.user_type)
    }
    let username = option_set
        .username
        .unwrap_or(format!("Player{}", Utc::now().timestamp_millis() & 1000));
    let session = Session::new(
        username.clone(),
        option_set
            .uuid
            .unwrap_or(uuids::get_offline_player_uuid(&username).to_string()),
        option_set.access_token,
        str_to_optional(option_set.xuid),
        str_to_optional(option_set.client_id),
        account_type.unwrap_or_default(),
    );
    let run_args = RunArgs::new(
        args::Network::new(session, proxy),
        super::WindowSettings::new(option_set.width, option_set.height),
        args::Directions::new(
            option_set.game_dir.clone(),
            option_set
                .resource_pack_dir
                .unwrap_or(format!("{}/resourcepacks", option_set.game_dir)),
            option_set
                .assets_dir
                .unwrap_or(format!("{}/assets", option_set.game_dir)),
            option_set.asset_index,
        ),
        args::Game::new(option_set.version, option_set.version_type),
    );
    bootstrap::initialize();
}

fn str_to_optional(string: String) -> Option<String> {
    if string.is_empty() {
        None
    } else {
        Some(string)
    }
}
