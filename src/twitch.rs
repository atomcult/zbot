use std;
use std::io::Write;
use std::default::Default;
use irc::client::prelude::*;
use irc::error::IrcError;
use config::Channel;

pub fn init(bot_user: String, bot_pass: String, owners: Vec<String>, chan: Channel) {

    // Set up IRC connection
    let cfg = Config {
        owners: Some(owners.clone()),
        nickname: Some(bot_user),
        password: Some(bot_pass),
        server: Some(String::from("irc.chat.twitch.tv")),
        port: Some(443),
        use_ssl: Some(true),
        channels: Some(vec!(format!("#{}", chan.name.clone()))),
        ..Default::default()
    };
    let s = IrcClient::from_config(cfg).unwrap();
    let s = match s.identify() {
        Ok(_) => s,
        Err(e) => panic!("Unable to identify to server: {}", e),
    };

    // Set up extra twitch irc capabilities
    s.send("CAP REQ :twitch.tv/membership").unwrap();
    s.send("CAP REQ :twitch.tv/tags").unwrap();
    s.send("CAP REQ :twitch.tv/commands").unwrap();

    // Open log file
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("twitch.log");
    let mut log = match log_file {
        Ok(f) => f,
        Err(e) => panic!("Error: {}", e),
    };


    // Main command processing loop
    s.for_each_incoming(|msg| {
        // Log the message
        let log_msg = log_format(msg.to_string());
        print!("{}", log_msg);
        let _ = log.write_all(log_msg.as_bytes());

        // Parse
        let Message { command, .. } = msg;
        match command {
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
    s.send(format!("PRIVMSG {un} :{}", msg, un = chan).as_str())
}

fn log_format (s: String) -> String {
    use std::time::SystemTime;
    if let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        format!("[{}] {}", time.as_secs(), s)
    } else {
        format!("[ERR] {}", s)
    }
}
