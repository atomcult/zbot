use std::fs::File;
use std::io::prelude::*;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub twitch: Option<Twitch>,
    pub discord: Option<Discord>,
}

impl Config {
    pub fn open() -> Config {
        let mut file = File::open("config").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        toml::from_str(&contents).unwrap()
    }
}

#[derive(Deserialize, Debug)]
pub struct Twitch {
    pub user: String,
    pub pass: String,
    pub owners: Vec<String>,
    pub channels: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Discord {
    pub token: String,
}
