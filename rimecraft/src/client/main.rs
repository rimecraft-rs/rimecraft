use crate::network::Proxy;
use clap::Parser;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

#[derive(Debug, Parser)]
struct OptionSet {
    server: Option<String>,
    #[arg(default_value_t = 25565)]
    port: i64,
    #[arg(default_value_t = format!("."))]
    game_dir: String,
    assets_dir: Option<String>,
    resource_pack_dir: Option<String>,
    proxy_host: Option<String>,
    #[arg(default_value_t = 8080)]
    proxy_port: u16,
    proxy_user: Option<String>,
    proxy_pass: Option<String>,
    #[arg(default_value_t = format!("Player 114514"))]
    username: String,
    uuid: Option<String>,
    #[arg(default_value_t = format!(""))]
    xuid: String,
    #[arg(default_value_t = format!(""))]
    client_id: String,
    #[arg(default_value_t = format!(""))]
    access_token: String,
    version: String,
    #[arg(default_value_t = 854)]
    width: i64,
    #[arg(default_value_t = 480)]
    height: i64,
    asset_index: Option<String>,
    #[arg(default_value_t = format!("legacy"))]
    user_type: String,
    #[arg(default_value_t = format!("release"))]
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
}
