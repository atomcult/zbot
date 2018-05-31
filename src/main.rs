#![feature(assoc_unix_epoch)]

extern crate irc;
extern crate toml;
#[macro_use] extern crate serde_derive;

mod auth;
mod config;
mod cmd;
mod twitch;

use std::thread;

fn main() {
    let cfg = config::Config::open();

    let mut threads = Vec::new();
    for (_, channel) in &cfg.channels {
        // Create local copies of variables
        let user = cfg.user.clone();
        let pass = cfg.pass.clone();
        let owners = cfg.owners.clone();
        let channel = channel.clone();
        
        // Spawn thread
        threads.push(thread::spawn(|| {
            twitch::init( user, pass, owners, channel);
        }));
    }

    for t in threads {
        t.join().unwrap_or_else(|_|());
    }
}
