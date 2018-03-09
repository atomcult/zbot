use std::env;
use serenity::client::Client;
use serenity::prelude::{EventHandler, Context};
use serenity::framework::standard::StandardFramework;
use serenity::model::gateway::Ready;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_game_name("Dev. Hell");
        ctx.idle();
    }
}

pub fn init() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .cmd("hello", hello)
        .cmd("kill", kill)
        .cmd("version", version));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

command!(hello(_context, message) {
    let _ = message.reply("Pong!");
    // println!("CONTEXT");
    // println!("{:#?}", _context);
    // println!("");
    println!("MESSAGE");
    println!("{:#?}", message);
});

command!(kill(context, _message) {
    context.quit();
});

command!(version(_context, message) {
    let _ = message.reply(&format!("{}", env!("GIT_VERSION")));
});