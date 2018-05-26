#![feature(assoc_unix_epoch)]
extern crate irc;
#[macro_use] extern crate serenity;
extern crate toml;
#[macro_use] extern crate serde_derive;

mod config;
mod cmd;
mod twitch;

use std::thread;

fn main() {
    let cfg = config::Config::open();

    // Initialize thread handles
    let mut twitch = None;

    // Spawn twitch thread
    if let Some(cfg) = cfg.twitch {
        twitch = Some(
            thread::spawn(|| twitch::init(cfg))
            );
    }

    // Join threads
    if let Some(handle) = twitch  { handle.join().unwrap(); }
}
