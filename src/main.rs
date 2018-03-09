extern crate irc;
#[macro_use] extern crate serenity;

mod discord;
mod twitch;

use std::thread;

fn main() {
    // Spawn threads
    let discord = thread::spawn(move || discord::init() );
    let twitch  = thread::spawn(move || twitch::init() );

    // Join child processes to parent
    let _ = discord.join();
    let _ = twitch.join();
}
