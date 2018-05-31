use std::{env,process};
use std::fs::File;
use std::collections::HashMap;
use std::io::prelude::*;
use toml;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub user: String,
    pub pass: String,
    pub owners: Vec<String>,
    pub channels: HashMap<String, Channel>,
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

#[serde(default)]
#[derive(Clone, Deserialize, Debug)]
pub struct Channel {
    pub name: String,
    pub cmd_prefix: char,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            name: String::from(""),
            cmd_prefix: '!',
        }
    }
}
