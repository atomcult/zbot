extern crate irc;
#[macro_use] extern crate serenity;

mod discord;
mod twitch;

use std::thread;

fn main() {
    let discord_child = thread::spawn(move || { discord::init(); });
    let twitch_child = thread::spawn(move || { twitch::init(); });

    let discord_res = discord_child.join();
    let twitch_res = twitch_child.join();

    println!("Discord exited.");
}
