extern crate irc;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate rb;
extern crate rusqlite;
extern crate rand;
extern crate regex;

mod auth;
mod config;
mod cmd;
mod state;
mod twitch;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

fn main() {
    // TODO: Add cli parsing
    // TODO: Changeable config dir

    // Set config path
    let mut cfg_path;
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        cfg_path = PathBuf::from(dir);
    } else {
        cfg_path = std::env::home_dir().unwrap();
        cfg_path.push(".config");
    }
    cfg_path.push("zbot");

    // If config path doesn't exist, create it
    if !cfg_path.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&cfg_path)
            .unwrap();
    }

    // If cfg_file doesn't exist, panic
    let mut cfg_file = cfg_path.clone();
    cfg_file.push("config.toml");
    if !cfg_file.exists() {
        println!(
            "Config file `{}` does not exist.",
            cfg_file.to_str().unwrap()
        );
        std::process::exit(1);
    }

    let cfg = config::Config::open(&cfg_file);
    let state = state::MainState::new();

    let mut threads = Vec::new();
    for channel in cfg.channels.values() {
        // Create local copies of variables
        let user = cfg.user.clone();
        let pass = cfg.pass.clone();
        let owners = cfg.owners.clone();
        let channel = channel.clone();
        let t_state = state::ThreadState::new(Arc::clone(&state));

        // Spawn thread
        threads.push(thread::spawn(move || {
            twitch::init(&t_state, &channel, &owners, &user, &pass);
        }));
    }

    loop {
        {
            // Lock state and check for shutdown
            let state = state.lock().unwrap();
            if state.shutdown {
                std::process::exit(0)
            }
        } // Unlock state
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
