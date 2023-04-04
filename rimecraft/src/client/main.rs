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
struct OptionSet {
    #[arg(long, default_value_t = format!("."))]
    game_dir: String,
    #[arg(long)]
    assets_dir: Option<String>,
    #[arg(long)]
    resource_pack_dir: Option<String>,
    #[arg(long)]
    proxy_host: Option<String>,
    #[arg(long, default_value_t = 8080)]
    proxy_port: u16,
    #[arg(long)]
    proxy_user: Option<String>,
    #[arg(long)]
    proxy_pass: Option<String>,
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    uuid: Option<String>,
    #[arg(long, default_value_t = format!(""))]
    xuid: String,
    #[arg(long, default_value_t = format!(""))]
    client_id: String,
    #[arg(long)]
    access_token: String,
    #[arg(long)]
    version: String,
    #[arg(long, default_value_t = 854)]
    width: u32,
    #[arg(long, default_value_t = 480)]
    height: u32,
    #[arg(long)]
    asset_index: Option<String>,
    #[arg(long, default_value_t = format!("legacy"))]
    user_type: String,
    #[arg(long, default_value_t = format!("release"))]
    version_type: String,
}

pub fn main() {
    crate::consts::create_game_version();
    let option_set = OptionSet::parse();
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
