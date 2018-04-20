use std::{env,process};
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
        // Open config.toml
        let mut file = File::open("config.toml").unwrap_or_else( |_| {
            if let Ok(dir) = env::current_dir() {
                println!("`{}/config.toml` does not exist.", dir.to_str().unwrap());
            } else {
                println!("Current directory does not exist, or you do not have sufficient permissions.");
            }

            process::exit(1);
        });

        // Read contents of config.toml and serialize
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
