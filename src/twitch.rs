use std;
use std::io::Write;
use std::default::Default;
use irc::client::prelude::*;
use irc::error::IrcError;
use irc::proto::message::Tag;

use auth::Auth;
use config::Channel;
use cmd;

pub fn init(bot_user: String, bot_pass: String, owners: Vec<String>, chan_cfg: Channel) {

    // Set up IRC config
    let cfg = Config {
        owners: Some(owners.clone()),
        nickname: Some(bot_user.clone()),
        password: Some(bot_pass.clone()),
        server: Some(String::from("irc.chat.twitch.tv")),
        port: Some(443),
        use_ssl: Some(true),
        channels: Some(vec!(format!("#{}", chan_cfg.name.to_lowercase()))),
        ..Default::default()
    };

    loop { // Start loop to handle twitch RECONNECTs
        let s = IrcClient::from_config(cfg.clone()).unwrap();
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

        // Create command buffer
        let mut cmd_buffer = cmd::CmdList::new();

        // Main command processing loop
        s.for_each_incoming(|msg| {
            // Log the message
            let log_msg = log_format(msg.to_string());
            print!("{}", log_msg);
            let _ = log.write_all(log_msg.as_bytes());

            // Parse
            let Message { command, tags, prefix } = msg;
            match command {
                Command::PING(_, None) => s.send("PONG :tmi.twitch.tv").unwrap(),
                Command::PRIVMSG(chan, mut cmd) => {
                    if cmd.remove(0) == chan_cfg.cmd_prefix {
                        let auth = eval_auth(tags, prefix, &owners);
                        let msg_list = cmd_buffer.exec(auth, &cmd);
                        if let Some(msgv) = msg_list {
                            for msg in &msgv {
                                chanmsg(&s, &chan, msg).unwrap();
                            }
                        }
                    }
                },
                Command::Raw(cmd, ..) => {
                    if cmd == "RECONNECT" {
                        &s.send_quit("");
                    }
                },
                _ => {},
            };
        }).unwrap();
    }
}

fn chanmsg(s: &IrcClient, chan: &str, msg: &str) -> Result<(), IrcError> {
    println!("SENDING >>> PRIVMSG {un} :{}\n", msg, un = chan);
    s.send_privmsg(chan, msg)
}

fn log_format(s: String) -> String {
    use std::time::SystemTime;
    if let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        format!("[{}] {}", time.as_secs(), s)
    } else {
        format!("[ERR] {}", s)
    }
}

fn eval_auth(tags: Option<Vec<Tag>>, prefix: Option<String>, owners: &[String]) -> Auth {
    if let Some(prefix) = prefix {
        let prefix: Vec<&str> = prefix.split('!').collect();
        let from_user = prefix[0];
        if let Some(tags) = tags {
            // Check if user is an owner
            for owner in owners.into_iter() { if from_user == owner.as_str() { return Auth::Owner } }

            // Otherwise get their auth from tags
            for Tag(key, val) in tags {
                if key == "badges" {
                    if let Some(val) = val {
                        if val.contains("broadcaster") {
                            return Auth::Streamer;
                        } else if val.contains("moderator") {
                            return Auth::Mod;
                        } else if val.contains("subscriber") {
                            return Auth::Subscriber;
                        }
                    }
                    break;
                }
            }
        }
    }
    Auth::Viewer
}
