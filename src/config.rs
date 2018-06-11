use std::{env, process};
use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::prelude::*;
use std::path::PathBuf;
use toml;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub user: String,
    pub pass: String,
    pub owners: Vec<String>,
    pub channels: HashMap<String, Channel>,
}

impl Config {
    pub fn open(path: &PathBuf) -> Config {
        // Open config.toml
        let mut file = File::open(&path).unwrap_or_else(|_| {
            if let Ok(dir) = env::current_dir() {
                println!("`{}/config.toml` does not exist.", dir.to_str().unwrap());
            } else {
                println!(
                    "Current directory does not exist, or you do not have sufficient permissions."
                );
            }

            process::exit(1);
        });

        // Read contents of config.toml and serialize
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut cfg: Config = toml::from_str(&contents).unwrap();

        // FIXME: Surely there's a better way to do this
        // Recreate each channel with config dir
        let mut channels = HashMap::new();
        for mut chan in cfg.channels.values() {
            let mut path = PathBuf::from(path.parent().unwrap());
            path.push(format!("data/{}", chan.name.to_lowercase()));
            if !path.exists() {
                DirBuilder::new().recursive(true).create(&path).unwrap();
            }

            channels.insert(
                chan.name.clone(),
                Channel {
                    dir: path,
                    name: chan.name.clone(),
                    cmd_prefix: chan.cmd_prefix,
                },
            );
        }
        cfg.channels = channels;
        cfg
    }
}

#[serde(default)]
#[derive(Clone, Deserialize, Debug)]
pub struct Channel {
    #[serde(skip)]
    pub dir: PathBuf,
    pub name: String,
    pub cmd_prefix: char,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            name: String::from(""),
            cmd_prefix: '!',
            dir: PathBuf::new(),
        }
    }
}
