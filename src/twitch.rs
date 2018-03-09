// use std::time::{Duration, Instant};
use std::default::Default;
use irc::client::prelude::*;
// use irc::client::data::user::User;
use irc::error::IrcError;

// struct BotCommand {
//     name: String,
//     timeout: Duration,
//     last_used: Option<Instant>,
//     function: fn(msg: &str),
// }

// impl BotCommand {
//     fn new(name: String, timeout: Duration, function: fn(msg: &str)) -> Self {
//         BotCommand {
//             name: name,
//             timeout: timeout,
//             last_used: None,
//             function: function,
//         }
//     }
// }

pub fn init() {
    // Set up config
    let bot_name = "zedexv";
    let bot_pass = "oauth:uxp0fl69kng0mngyquhialj37fqgqm";
    let chans = vec![format!("#zedexv"), format!("#pqplays")];

    // TODO: Set up command array

    // Set up IRC connection
    let cfg = Config {
        owners: Some(vec![format!("zedexv")]),
        nickname: Some(bot_name.to_string()),
        password: Some(bot_pass.to_string()),
        server: Some(format!("irc.twitch.tv")),
        port: Some(6667),
        use_ssl: Some(false),
        channels: Some(chans),
        ..Default::default()
    };
    let s = IrcClient::from_config(cfg).unwrap();
    let s = match s.identify() {
        Ok(_) => s,
        Err(e) => panic!("Unable to identify to server: {}", e),
    };

    // Set up extra twitch irc capabilities
    s.send("CAP REQ :twitch.tv/membership  ").unwrap();
    s.send("CAP REQ :twitch.tv/tags  ").unwrap();
    s.send("CAP REQ :twitch.tv/commands  ").unwrap();

    // Main command processing loop
    s.for_each_incoming(|msg| {
        println!("{}", msg.to_string());
        let Message {
            tags: tags,
            prefix: _,
            command: cmd,
        } = msg;
        match cmd {
            Command::PING(server, None) => s.send(format!("PONG {}", server).as_str()).unwrap(),
            Command::PRIVMSG(chan, cmd) => {
                if cmd == "!pyramid" {
                    chanmsg(&s, &chan, "ffYeah!").unwrap();
                    chanmsg(&s, &chan, "CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "ffYeah! CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "CenaWins ffYeah! CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "ffYeah! CenaWins ffYeah! CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "CenaWins ffYeah! CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "ffYeah! CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "CenaWins ffYeah!").unwrap();
                    chanmsg(&s, &chan, "ffYeah!").unwrap();
                }
            }
            _ => {}
        };
    }).unwrap();
}

fn chanmsg(s: &IrcClient, chan: &str, msg: &str) -> Result<(), IrcError> {
    println!("SENDING >>> PRIVMSG {un} :{}\n", msg, un = chan);
    s.send(format!("PRIVMSG {un} :{}  ", msg, un = chan).as_str())
}
