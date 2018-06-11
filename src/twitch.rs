

use auth::Auth;
use cmd;
use config::Channel;
use irc::client::prelude::*;
use irc::error::IrcError;
use irc::proto::message::Tag;
use rb::*;
use rusqlite::Connection;
use state::ThreadState;
use std;
use std::default::Default;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn init(
    state: &Arc<Mutex<ThreadState>>,
    chan_cfg: &Channel,
    owners: &[String],
    bot_user: &str,
    bot_pass: &str,
) {

    // Set up IRC config
    let cfg = Config {
        owners: Some(owners.to_owned()),
        nickname: Some(bot_user.to_string()),
        password: Some(bot_pass.to_string()),
        server: Some(String::from("irc.chat.twitch.tv")),
        port: Some(443),
        use_ssl: Some(true),
        channels: Some(vec![format!("#{}", chan_cfg.name.to_lowercase())]),
        ..Default::default()
    };

    // Open log file
    let mut log_path = chan_cfg.dir.clone();
    log_path.push("log");
    let log_file = std::fs::OpenOptions::new().create(true).append(true).open(
        log_path,
    );
    let mut log = match log_file {
        Ok(f) => f,
        Err(e) => panic!("Error: {}", e),
    };

    // Open SQLite connection
    let mut db_path = chan_cfg.dir.clone();
    db_path.push("db");
    let db = Connection::open(db_path).unwrap();

    // Try to create tables
    let _ = db.execute(
        "CREATE TABLE quote (
                        id       INTEGER PRIMARY KEY,
                        quote    TEXT NOT NULL
                        )",
        &[],
    );

    let _ = db.execute(
        "CREATE TABLE alias (
                        id         INTEGER PRIMARY KEY,
                        alias      TEXT NOT NULL,
                        command    TEXT NOT NULL
                        )",
        &[],
    );

    {
        // Add db to ThreadState
        let mut state = state.lock().unwrap();
        state.db = Some(db);
    }

    // Create command buffer
    let mut cmd_list = cmd::CmdList::new();
    let mut send_buffer: SpscRb<Option<Instant>> = SpscRb::new(100);

    loop {
        // Start loop to handle twitch RECONNECTs
        let s = IrcClient::from_config(cfg.clone()).unwrap();
        let s = match s.identify() {
            Ok(_) => s,
            Err(e) => panic!("Unable to identify to server: {}", e),
        };

        // Set up extra twitch irc capabilities
        s.send("CAP REQ :twitch.tv/membership").unwrap();
        s.send("CAP REQ :twitch.tv/tags").unwrap();
        s.send("CAP REQ :twitch.tv/commands").unwrap();

        // Main command processing loop
        s.for_each_incoming(|msg| {
            // Clone ref to state
            let state = Arc::clone(&state);

            // Log the message
            let log_msg = log_format(&msg.to_string());
            print!("{}", log_msg);
            let _ = log.write_all(log_msg.as_bytes());

            // Parse
            let Message {
                command,
                tags,
                prefix,
            } = msg;
            match command {
                Command::PING(_, None) => s.send("PONG :tmi.twitch.tv").unwrap(),
                Command::PRIVMSG(chan, mut cmd) => {
                    if cmd.remove(0) == chan_cfg.cmd_prefix {
                        let context = Context::new(&chan_cfg.name, tags, prefix, &owners);
                        let msgv = cmd_list.exec(state, &context, &cmd);
                        send_msg(&s, &mut send_buffer, &chan, msgv);
                    }
                }
                Command::Raw(cmd, ..) => {
                    if cmd == "RECONNECT" {
                        let _ = s.send_quit("");
                    }
                }
                _ => {}
            };
        }).unwrap();
    }
}

pub struct Context {
    pub sender: String,
    pub channel: String,
    pub auth: Auth,
    pub tags: Option<Vec<Tag>>,
    pub prefix: Option<String>,
}

impl Context {
    pub fn new(
        channel: &str,
        tags: Option<Vec<Tag>>,
        prefix: Option<String>,
        owners: &[String],
    ) -> Self {
        let sender = Self::user_from_prefix(&prefix);
        let auth = Self::eval_auth(&tags, &sender, owners);
        Self {
            sender,
            channel: String::from(channel),
            auth,
            tags,
            prefix,
        }
    }

    pub fn get_sender_display(&self) -> Option<String> {
        if let Some(tags) = &self.tags {
            for Tag(key, val) in tags {
                if key == "display-name" {
                    if let Some(val) = val {
                        let display_name = val.clone();
                        return Some(display_name);
                    }
                }
            }
            None
        } else {
            None
        }
    }

    fn user_from_prefix(prefix: &Option<String>) -> String {
        let prefix = prefix.clone().unwrap();
        let prefix: Vec<&str> = prefix.split('!').collect();
        String::from(prefix[0])
    }

    fn eval_auth(tags: &Option<Vec<Tag>>, sender: &str, owners: &[String]) -> Auth {
        if let Some(tags) = tags {
            // Check if user is an owner
            for owner in owners {
                if sender == owner.as_str() {
                    return Auth::Owner;
                }
            }

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
        Auth::Viewer
    }
}

fn chanmsg(s: &IrcClient, chan: &str, msg: &str) -> std::result::Result<(), IrcError> {
    println!("SENDING >>> PRIVMSG {un} :{}\n", msg, un = chan);
    s.send_privmsg(chan, msg)
}

fn log_format(s: &str) -> String {
    use std::time::SystemTime;
    if let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        format!("[{}] {}", time.as_secs(), s)
    } else {
        format!("[ERR] {}", s)
    }
}

fn send_msg(
    s: &IrcClient,
    send_buffer: &mut SpscRb<Option<Instant>>,
    chan: &str,
    msgv: Option<Vec<String>>,
) {
    if let Some(msgv) = msgv {
        let (prod, cons) = (send_buffer.producer(), send_buffer.consumer());

        let mut purge_count = 0;
        let mut buffer = [None; 100];
        if cons.get(&mut buffer).is_ok() {
            for inst in buffer.into_iter() {
                if let Some(inst) = inst {
                    if inst.elapsed().as_secs() >= 30 {
                        purge_count += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        if purge_count == 100 {
            cons.skip_pending().unwrap();
        } else if purge_count > 0 {
            cons.skip(purge_count).unwrap();
        }

        if send_buffer.slots_free() >= msgv.len() {
            let mut instv = Vec::new();
            for _ in &msgv {
                instv.push(Some(Instant::now()));
            }
            prod.write(&instv).unwrap();
            for msg in msgv {
                chanmsg(s, chan, &msg).unwrap();
            }
        }
    }
}
