use serde::Deserialize;
use std::{fs, sync::OnceLock};

const KEY_RIGHTALT: u32 = 100;
const KEY_LEFTALT: u32 = 56;
const KEY_RIGHTCTRL: u32 = 97;
const KEY_LEFTCTRL: u32 = 29;
pub const MOUSE_LEFT: u32 = 272;

#[derive(Deserialize)]
pub struct Config {
    pub language: String,
    pub modkey: String,
    pub vendor: String,
    pub baidu: Option<Baidu>,
    pub google: Option<Google>,
    pub bing: Option<Bing>,
}

#[derive(Deserialize, Clone)]
pub struct Baidu {
    pub appid: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct Google {}

#[derive(Deserialize)]
pub struct Bing {}

pub fn get_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let config = fs::read_to_string("config.toml").unwrap();
        let config = toml::from_str(config.as_str()).unwrap();
        config
    })
}
pub fn key2no(key: &str) -> u32 {
    match key {
        "LEFTALT" => KEY_LEFTALT,
        "RIGHTALT" => KEY_RIGHTALT,
        "LEFTCTRL" => KEY_LEFTCTRL,
        "RIGHTCTRL" => KEY_RIGHTCTRL,
        _ => 0,
    }
}
