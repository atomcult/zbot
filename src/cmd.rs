use auth::Permissions;
use rand::distributions::Uniform;
use rand::prelude::*;
use regex::Regex;
use rusqlite::Connection;
use state::ThreadState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use twitch::Context;
use strawpoll;

pub struct CmdList {
    commands: HashMap<&'static str, Cmd>,
}

impl CmdList {
    pub fn new() -> Self {
        let mut commands = HashMap::new();

        commands.insert("aliasmod", mod_alias());

        commands.insert("null", null());
        commands.insert("quote", quote());
        commands.insert("quoteadd", quoteadd());
        commands.insert("quoterm", quoterm());
        commands.insert("say", say());
        commands.insert("thicc", thicc());
        commands.insert("tiny", tinytext());
        commands.insert("smol", smallcaps());
        commands.insert("numberwang", numberwang());
        commands.insert("8ball", eightball());
        commands.insert("flipcoin", coinflip());
        commands.insert("tcount", tcount());
        commands.insert("roll", roll());
        commands.insert("count", count());
        commands.insert("version", version());
        commands.insert("shutdown", shutdown());

        commands.insert("strawpoll", strawpoll_cmd());

        Self { commands }
    }

    pub fn exec(
        &mut self,
        state: Arc<Mutex<ThreadState>>,
        context: &Context,
        command: &str,
    ) -> Option<Vec<String>> {
        let (cmd, args) = pop_cmd(command);
        if cmd == "alias" {
            if context.auth.intersects(Permissions::Streamer | Permissions::Mod) {
                if let Some(args) = args {
                    let (alias, command) = pop_cmd(&args);
                    let state = state.lock().unwrap();
                    if let Some(db) = &state.db {
                        if let Some(mut command) = command {
                            let mut auth_mod = String::new();
                            while command.starts_with('+') || command.starts_with('-') {
                                let (new_mod, new_cmd) = pop_cmd(&command);
                                auth_mod.push_str(&new_mod);
                                command = match new_cmd {
                                    Some(cmd) => cmd,
                                    None => return None,
                                };
                            }
                            let (cmd, _) = pop_cmd(&command);
                            if let Some(cmd) = self.commands.get(cmd.as_str()) {
                                // Make sure that the user who's aliasing has permission to use the
                                // command being aliased
                                if context.auth.intersects(cmd.auth) {
                                    let mut auth = cmd.auth.clone();
                                    auth.set(Permissions::ReadOnly, true);
                                    let mut attr_val = true;
                                    let mut attr;
                                    for ch in auth_mod.chars() {
                                        match ch {
                                            '+' => { attr_val = true;  continue; },
                                            '-' => { attr_val = false; continue; },
                                            'r' => attr = Permissions::ReadOnly,
                                            'o' => attr = Permissions::Owner,
                                            'b' => attr = Permissions::Streamer,
                                            'm' => attr = Permissions::Mod,
                                            's' => attr = Permissions::Sub,
                                            'v' => attr = Permissions::Viewer,
                                            _ => continue,
                                        }
                                        auth.set(attr, attr_val);
                                    }
                                    rm_alias(&db, &alias);
                                    add_alias(&db, &alias, &auth, &command);
                                }
                            }
                        } else {
                            rm_alias(&db, &alias)
                        }
                    }
                    return None;
                } else {
                    let msgv = Some(vec![String::from("Usage: !alias <alias> [auth] <cmd> [args...]")]);
                    return msgv;
                }
            } else {
                None
            }
        } else {
            let mut msgv = None;

            // Search for alias
            let mut alias_res = None;
            {
                let state = state.lock().unwrap();
                if let Some(db) = &state.db {
                    alias_res = get_alias(&db, &cmd);
                }
            }

            // Search for alias and exec
            if let Some((alias_auth, alias_cmd)) = alias_res {
                let (c, mut alias_args) = pop_cmd(&alias_cmd);

                if !alias_auth.contains(Permissions::ReadOnly) {
                    if let Some(args) = args {
                        alias_args = if alias_args.is_some() {
                            let mut tmp_args = alias_args.unwrap();
                            tmp_args.push(' ');
                            tmp_args.push_str(&args);
                            Some(tmp_args)
                        } else {
                            Some(args)
                        };
                    }
                }

                if let Some(c) = self.commands.get(c.as_str()) {
                    if context.auth.intersects(alias_auth) {
                        msgv = c.exec(state, &context, alias_args);
                    }
                }
            }
            // Else search for command and exec
            else if let Some(c) = self.commands.get(&cmd.as_str()) {
                if context.auth.intersects(c.auth) {
                    msgv = c.exec(state, &context, args);
                }
            }
            msgv
        }
    }
}

pub struct Cmd {
    func: fn(t_state: Arc<Mutex<ThreadState>>,
             context: &Context,
             Option<String>)
             -> Option<Vec<String>>,
    pub bucket: Option<Bucket>,
    pub auth: Permissions,
}

impl Cmd {
    pub fn exec(
        &self,
        t_state: Arc<Mutex<ThreadState>>,
        context: &Context,
        args: Option<String>,
    ) -> Option<Vec<String>> {
        (self.func)(t_state, context, args)
    }
}

pub struct Bucket {
    pub count: u32,
    pub interval: Duration,
}


////////////////////////////////////////////////////////////////////////////////////////////////////
//                                          Bot Commands                                          //
////////////////////////////////////////////////////////////////////////////////////////////////////

fn mod_alias() -> Cmd {
    Cmd {
        func: |t_state, _, args| {
            if let Some(args) = args {
                let t_state = t_state.lock().unwrap();
                if let Some(db) = &t_state.db {
                    let (alias, args) = pop_cmd(&args);
                    if let Some(args) = args {
                        if let Some((auth, _)) = get_alias(&db, &alias) {
                            let mut auth = auth;
                            let mut attr_val = true;
                            let mut attr;
                            for ch in args.chars() {
                                match ch {
                                    '+' => { attr_val = true;  continue; },
                                    '-' => { attr_val = false; continue; },
                                    'r' => attr = Permissions::ReadOnly,
                                    'o' => attr = Permissions::Owner,
                                    'b' => attr = Permissions::Streamer,
                                    'm' => attr = Permissions::Mod,
                                    's' => attr = Permissions::Sub,
                                    'v' => attr = Permissions::Viewer,
                                    _ => continue,
                                }
                                auth.set(attr, attr_val);
                            }
                            let auth: u8 = auth.bits();
                            db.execute("UPDATE alias SET auth=(?1) WHERE alias=?2", &[&auth, &alias])
                                .unwrap();
                        }
                    }
                }
            }
            None
        },
        bucket: None,
        auth: Permissions::Streamer | Permissions::Mod,
    }
}

fn null() -> Cmd {
    Cmd {
        func: |_, _, _| None,
        bucket: None,
        auth: Permissions::Streamer,
    }
}

fn say() -> Cmd {
    Cmd {
        func: |_, _, args| if let Some(args) = args {
            Some(vec![args])
        } else {
            None
        },
        bucket: None,
        auth: Permissions::Streamer | Permissions::Mod,
    }
}

fn count() -> Cmd {
    Cmd {
        func: |_, _, args| if let Some(args) = args {
            match args.parse::<u32>() {
                Ok(u) => {
                    let mut v = Vec::new();
                    for i in 0..u {
                        v.push(format!("{}", i));
                    }
                    Some(v)
                }
                Err(_) => None,
            }
        } else {
            None
        },
        bucket: None,
        auth: Permissions::Owner,
    }
}

fn strawpoll_cmd() -> Cmd {
    Cmd {
        func: |t_state, _, args| {
            if let Some(args) = args {
                let mut title: &str = "";
                let mut options: Vec<&str> = Vec::new();
                let splits = args.split('|');

                let mut is_title = true;
                for split in splits {
                    if is_title {
                        title = split.trim();
                        is_title = false;
                    } else {
                        options.push(split.trim())
                    }
                }
                if let Ok(poll) = strawpoll::create_poll(title, &options) {
                    if let Ok(mut t_state) = t_state.lock() {
                        t_state.poll_id = Some(poll.id);
                    }
                    return Some(vec![format!("https://strawpoll.me/{}", poll.id)])
                }
            } else {
                if let Ok(t_state) = t_state.lock() {
                    if let Some(poll_id) = t_state.poll_id {
                        if let Ok(poll) = strawpoll::get_poll(poll_id) {
                            if let Some(votes) = poll.votes {
                                let mut s = format!("\"{}\" (https://strawpoll.me/{}): ", poll.title, poll.id);
                                for i in 0..poll.options.len() {
                                    s.push_str(&format!("\"{}\": {} votes", poll.options[i], votes[i]));
                                    if i != poll.options.len()-1 {
                                        s.push_str(", ");
                                    }
                                }
                                return Some(vec![s])
                            } else {
                                return Some(vec![format!("https://strawpoll.me/{}", poll_id)])
                            }
                        }
                    }
                }
            }
            None
        },
        bucket: None,
        auth: Permissions::Streamer | Permissions::Mod,
    }
}

fn thicc() -> Cmd {
    Cmd {
        func: |_, _, args| if let Some(arg) = args {
            let mut response = String::new();
            for letter in arg.chars() {
                response.push(match letter {
                    'a' => '卂',
                    'b' => '乃',
                    'c' => '匚',
                    'd' => '刀',
                    'e' => '乇',
                    'f' => '下',
                    'g' => '厶',
                    'h' => '卄',
                    'i' => '工',
                    'j' => '丁',
                    'k' => '长',
                    'l' => '乚',
                    'm' => '从',
                    'n' => '𠘨',
                    'o' => '口',
                    'p' => '尸',
                    'q' => '㔿',
                    'r' => '尺',
                    's' => '丂',
                    't' => '丅',
                    'u' => '凵',
                    'v' => 'リ',
                    'w' => '山',
                    'x' => '乂',
                    'y' => '丫',
                    'z' => '乙',
                    'A' => '卂',
                    'B' => '乃',
                    'C' => '匚',
                    'D' => '刀',
                    'E' => '乇',
                    'F' => '下',
                    'G' => '厶',
                    'H' => '卄',
                    'I' => '工',
                    'J' => '丁',
                    'K' => '长',
                    'L' => '乚',
                    'M' => '从',
                    'N' => '𠘨',
                    'O' => '口',
                    'P' => '尸',
                    'Q' => '㔿',
                    'R' => '尺',
                    'S' => '丂',
                    'T' => '丅',
                    'U' => '凵',
                    'V' => 'リ',
                    'W' => '山',
                    'X' => '乂',
                    'Y' => '丫',
                    'Z' => '乙',
                    x => x,
                });
            }
            Some(vec![response])
        } else {
            None
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn tinytext() -> Cmd {
    Cmd {
        func: |_, _, args| if let Some(arg) = args {
            let mut response = String::new();
            for letter in arg.chars() {
                response.push(match letter {
                    'a' => 'ᵃ',
                    'b' => 'ᵇ',
                    'c' => 'ᶜ',
                    'd' => 'ᵈ',
                    'e' => 'ᵉ',
                    'f' => 'ᶠ',
                    'g' => 'ᵍ',
                    'h' => 'ʰ',
                    'i' => 'ᶦ',
                    'j' => 'ʲ',
                    'k' => 'ᵏ',
                    'l' => 'ˡ',
                    'm' => 'ᵐ',
                    'n' => 'ⁿ',
                    'o' => 'ᵒ',
                    'p' => 'ᵖ',
                    'q' => 'ᑫ',
                    'r' => 'ʳ',
                    's' => 'ˢ',
                    't' => 'ᵗ',
                    'u' => 'ᵘ',
                    'v' => 'ᵛ',
                    'w' => 'ʷ',
                    'x' => 'ˣ',
                    'y' => 'ʸ',
                    'z' => 'ᶻ',
                    'A' => 'ᴬ',
                    'B' => 'ᴮ',
                    'C' => 'ᶜ',
                    'D' => 'ᴰ',
                    'E' => 'ᴱ',
                    'F' => 'ᶠ',
                    'G' => 'ᴳ',
                    'H' => 'ᴴ',
                    'I' => 'ᴵ',
                    'J' => 'ᴶ',
                    'K' => 'ᴷ',
                    'L' => 'ᴸ',
                    'M' => 'ᴹ',
                    'N' => 'ᴺ',
                    'O' => 'ᴼ',
                    'P' => 'ᴾ',
                    'Q' => 'Q',
                    'R' => 'ᴿ',
                    'S' => 'ˢ',
                    'T' => 'ᵀ',
                    'U' => 'ᵁ',
                    'V' => 'ⱽ',
                    'W' => 'ᵂ',
                    'X' => 'ˣ',
                    'Y' => 'ʸ',
                    'Z' => 'ᶻ',
                    x => x,
                });
            }
            Some(vec![response])
        } else {
            None
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn smallcaps() -> Cmd {
    Cmd {
        func: |_, _, args| if let Some(arg) = args {
            let mut response = String::new();
            for letter in arg.chars() {
                response.push(match letter {
                    'a' => 'ᴀ',
                    'b' => 'ʙ',
                    'c' => 'ᴄ',
                    'd' => 'ᴅ',
                    'e' => 'ᴇ',
                    'f' => 'ғ',
                    'g' => 'ɢ',
                    'h' => 'ʜ',
                    'i' => 'ɪ',
                    'j' => 'ᴊ',
                    'k' => 'ᴋ',
                    'l' => 'ʟ',
                    'm' => 'ᴍ',
                    'n' => 'ɴ',
                    'o' => 'ᴏ',
                    'p' => 'ᴘ',
                    'q' => 'ǫ',
                    'r' => 'ʀ',
                    's' => 's',
                    't' => 'ᴛ',
                    'u' => 'ᴜ',
                    'v' => 'ᴠ',
                    'w' => 'ᴡ',
                    'x' => 'x',
                    'y' => 'ʏ',
                    'z' => 'ᴢ',
                    'A' => 'A',
                    'B' => 'B',
                    'C' => 'C',
                    'D' => 'D',
                    'E' => 'E',
                    'F' => 'F',
                    'G' => 'G',
                    'H' => 'H',
                    'I' => 'I',
                    'J' => 'J',
                    'K' => 'K',
                    'L' => 'L',
                    'M' => 'M',
                    'N' => 'N',
                    'O' => 'O',
                    'P' => 'P',
                    'Q' => 'Q',
                    'R' => 'R',
                    'S' => 'S',
                    'T' => 'T',
                    'U' => 'U',
                    'V' => 'V',
                    'W' => 'W',
                    'X' => 'X',
                    'Y' => 'Y',
                    'Z' => 'Z',
                    x => x,
                });
            }
            Some(vec![response])
        } else {
            None
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn numberwang() -> Cmd {
    Cmd {
        func: |_, _, arg| {
            if let Some(arg) = arg {
                if arg.parse::<f32>().is_ok() {
                    let mut rng = thread_rng();
                    let answers = vec![
                        "That's Numberwang!",
                        "Das ist Nümberwang!",
                        "Mmm... Yumberwang!",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Yes, that is a number.",
                        "Ja, das ist eine Nummer.",
                        "Ja, das ist eine Nummer.",
                        "Ja, das ist eine Nummer.",
                        "Ja, das ist eine Nummer.",
                    ];
                    return Some(vec![String::from(answers[rng.gen_range(0, answers.len())])])
                }
            }
            None
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn eightball() -> Cmd {
    Cmd {
        func: |_, _, _| {
            let mut rng = thread_rng();
            let answers = vec![
                "It is certain.",
                "It is decidedly so.",
                "Without a doubt.",
                "Yes, definitely.",
                "You may rely on it.",
                "As I see it, yes.",
                "Most likely.",
                "Outlook good.",
                "Yes.",
                "Signs point to yes.",
                "Don't count on it.",
                "My reply is no.",
                "My sources say no.",
                "Outlook not so good.",
                "Very doubtful.",
            ];
            Some(vec![String::from(answers[rng.gen_range(0, answers.len())])])
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn coinflip() -> Cmd {
    Cmd {
        func: |_, _, args| if let Some(args) = args {
            let iter = args.parse::<u8>();
            if let Ok(iter) = iter {
                let mut r = String::new();
                for _ in 0..iter as usize {
                    if random() {
                        r.push('H');
                    } else {
                        r.push('T');
                    }
                }
                return Some(vec![r]);
            } else {
                return None;
            }
        } else if random() {
            return Some(vec![String::from("Heads")]);
        } else {
            return Some(vec![String::from("Tails")]);
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn roll() -> Cmd {
    Cmd {
        func: |_, _, args| {
            let mut rng = thread_rng();
            if let Some(args) = args {
                let argv: Vec<&str> = args.split_whitespace().collect();

                let re_roll = Regex::new(r"^(\d+)?d(\d+)(:?[+-](\d+))?$").unwrap();
                let re_mod = Regex::new(r"^[+-]?\d+$").unwrap();

                let mut sum = 0;
                let mut sign = 1;
                for arg in argv {
                    if re_roll.is_match(arg) {
                        let caps = re_roll.captures(arg).unwrap();

                        let num_rolls;
                        let num_die_faces;
                        match caps.get(1) {
                            Some(m) => num_rolls = m.as_str().parse::<i64>().unwrap(),
                            None => num_rolls = 1,
                        }
                        num_die_faces = caps.get(2).unwrap().as_str().parse::<i64>().unwrap();
                        if let Some(m) = caps.get(3) {
                            sum += m.as_str().parse::<i64>().unwrap()
                        }

                        // If more than one iteration, create generator so that
                        // distribution is flat, and perform sum.
                        if num_rolls > 1 {
                            let between = Uniform::from(1..(num_die_faces + 1));
                            for _ in 0..num_rolls {
                                sum += sign * rng.sample(&between);
                            }
                        } else {
                            sum += sign * rng.gen_range::<i64>(1, num_die_faces + 1);
                        }
                    } else if re_mod.is_match(arg) {
                        sum += sign * arg.parse::<i64>().unwrap();
                    } else if arg == "+" {
                        sign = 1;
                    } else if arg == "-" {
                        sign = -1;
                    } else {
                        return None;
                    }
                }
                Some(vec![format!("{}", sum)])
            } else {
                let roll = rng.gen_range(1, 20);
                let mut roll_string = format!("{}", roll);
                if roll_string == "20" {
                    roll_string.push_str(" PogChamp");
                } else if roll_string == "1" {
                    roll_string.push_str(" NotLikeThis");
                }
                Some(vec![roll_string])
            }
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn tcount() -> Cmd {
    Cmd {
        func: |_, context, _| {
            let sender = &context.sender;
            let display = context.get_sender_display().unwrap();
            // hash username
            let mut hash: u64 = 0;
            for b in sender.as_bytes() {
                hash += u64::from(*b);
                hash += u64::from(b << 10);
                hash ^= u64::from(b >> 6);
            }
            hash += hash << 3;
            hash ^= hash >> 11;
            hash += hash << 15;

            let tcount = (hash % 101) as u8;
            Some(vec![format!("{}: {}/100", display, tcount)])
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn quote() -> Cmd {
    Cmd {
        func: |t_state, _, args| {
            let t_state = t_state.lock().unwrap();
            if let Some(db) = &t_state.db {
                let mut quote;
                if let Some(args) = args {
                    if let Ok(i) = args.parse::<u32>() {
                        quote = db.query_row(
                            "SELECT * FROM quote WHERE id=?1;",
                            &[&i],
                            |row| {
                                let id: u32 = row.get(0);
                                let q: String = row.get(1);
                                format!("[{}] {}", id, q)
                            },
                            );
                    } else {
                        return None
                    }
                } else {
                    quote = db.query_row(
                        "SELECT * FROM quote ORDER BY RANDOM() LIMIT 1;",
                        &[],
                        |row| {
                            let id: u32 = row.get(0);
                            let q: String = row.get(1);
                            format!("[{}] {}", id, q)
                        },
                        );
                }
                if let Ok(quote) = quote {
                    let mut v = Vec::new();
                    v.push(quote);
                    return Some(v);
                }
            }
            None
        },
        bucket: None,
        auth: Permissions::Viewer,
    }
}

fn quoteadd() -> Cmd {
    Cmd {
        func: |t_state, _, args| {
            if let Some(args) = args {
                let t_state = t_state.lock().unwrap();
                if let Some(db) = &t_state.db {
                    db.execute("INSERT INTO quote (quote) values (?1)", &[&args])
                        .unwrap();
                    let msg = db.query_row("SELECT * FROM quote ORDER BY id DESC LIMIT 1;", &[], |row| {
                        let id: u32 = row.get(0);
                        format!("Quote #{} added.", id)
                    });
                    if let Ok(msg) = msg {
                        return Some(vec![msg]);
                    }
                }
            }
            None
        },
        bucket: None,
        auth: Permissions::Streamer | Permissions::Mod,
    }
}

fn quoterm() -> Cmd {
    Cmd {
        func: |t_state, _, args| {
            if let Some(args) = args {
                if let Ok(i) = args.parse::<u32>() {
                    if i > 0 {
                        let t_state = t_state.lock().unwrap();
                        if let Some(db) = &t_state.db {
                            let id = format!("{}", i);
                            db.execute("DELETE FROM quote WHERE id=?1", &[&id]).unwrap();
                        }
                    }
                }
            }
            None
        },
        bucket: None,
        auth: Permissions::Streamer | Permissions::Mod,
    }
}

fn shutdown() -> Cmd {
    Cmd {
        func: |t_state, _, _| {
            let t_state = t_state.lock().unwrap();
            let mut state = t_state.main.lock().unwrap();
            state.shutdown = true;
            None
        },
        bucket: None,
        auth: Permissions::Owner,
    }
}

fn version() -> Cmd {
    Cmd {
        func: |_, _, _| {
            let v = env!("GIT_VERSION");
            Some(vec![String::from(v)])
        },
        bucket: None,
        auth: Permissions::Owner,
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
//                                        Helper Functions                                        //
////////////////////////////////////////////////////////////////////////////////////////////////////


fn pop_cmd(s: &str) -> (String, Option<String>) {
    let s = String::from(s.trim_left());
    let argv: Vec<&str> = s.splitn(2, ' ').collect();

    if argv.len() == 2 {
        (argv[0].to_string(), Some(argv[1].trim().to_string()))
    } else {
        (argv[0].to_string(), None)
    }
}

fn rm_alias(db: &Connection, alias: &str) {
    let _ = db.execute("DELETE FROM alias WHERE alias=?1", &[&alias]);
}

fn add_alias(db: &Connection, alias: &str, auth: &Permissions, cmd: &str) {
    let _ = db.execute(
        "INSERT INTO alias (auth, alias, command) VALUES (?1, ?2, ?3)",
        &[&auth.bits(), &alias, &cmd],
    );
}

fn get_alias(db: &Connection, alias: &str) -> Option<(Permissions, String)> {
    if let Ok((auth, cmd)) = db.query_row("SELECT * FROM alias WHERE alias=?1", &[&alias], |row| {
        let auth: u8 = row.get(1);
        let cmd: String = row.get(3);
        (auth, cmd)
    }) {
        let auth = Permissions::from_bits(auth).unwrap();
        Some((auth, cmd))
    } else {
        None
    }
}
