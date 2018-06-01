#![feature(assoc_unix_epoch)]

extern crate irc;
extern crate toml;
#[macro_use] extern crate serde_derive;
extern crate rb;

mod auth;
mod config;
mod cmd;
mod state;
mod twitch;

use std::thread;
use std::sync::Arc;

fn main() {
    // TODO: Add cli parsing
    // TODO: Changeable config dir
    let cfg = config::Config::open();
    let state = state::MainState::new();

    let mut threads = Vec::new();
    for (_, channel) in &cfg.channels {
        // Create local copies of variables
        let user = cfg.user.clone();
        let pass = cfg.pass.clone();
        let owners = cfg.owners.clone();
        let channel = channel.clone();
        let t_state = state::ThreadState::new(Arc::clone(&state));
        
        // Spawn thread
        threads.push(thread::spawn(|| {
            twitch::init(user, pass, owners, channel, t_state);
        }));
    }

    loop {
        { // Lock state and check for shutdown
            let state = state.lock().unwrap();
            if state.shutdown { std::process::exit(0) }
        } // Unlock state
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
