use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use rusqlite::Connection;
use rand::prelude::*;
use rand::distributions::Uniform;
use regex::Regex;

use auth::Auth;
use twitch::Context;
use state::ThreadState;

pub struct CmdList {
    commands: HashMap<&'static str, Cmd>,
}

impl CmdList {
    pub fn new() -> Self {
        let mut commands = HashMap::new();

        commands.insert("quote", quote());
        commands.insert("quoteadd", quoteadd());
        commands.insert("quoterm", quoterm());
        commands.insert("say", say());
        commands.insert("thicc", thicc());
        commands.insert("8ball", eightball());
        commands.insert("flipcoin", coinflip());
        commands.insert("roll", roll());
        commands.insert("count", count());
        commands.insert("version", version());
        commands.insert("shutdown", shutdown());

        Self {
            commands,
        }
    }

    pub fn exec(&mut self, state: Arc<Mutex<ThreadState>>, context: Context, command: &str) -> Option<Vec<String>> {
        let (cmd, args) = pop_cmd(command.to_string());
        if cmd == "alias" {
            if context.auth >= Auth::Mod {
                if let Some(args) = args {
                    let (alias, command) = pop_cmd(args);
                    let state = state.lock().unwrap();
                    if let Some(db) = &state.db {
                        rm_alias(&db, &alias);
                        if let Some(command) = command {
                            // If alias is the same as a command name, do not add the alias
                            if let Some(_) = self.commands.get(alias.as_str()){
                                let msgv = Some(vec!(format!("Cannot alias `{}`. Command with the same name exists already.", alias)));
                                return msgv;
                            }
                            add_alias(&db, &alias, &command);
                        }
                    }
                    return None
                } else {
                    let msgv = Some(vec!(String::from("Usage: !alias <alias> <cmd> [args...]")));
                    return msgv;
                }
            } else { return None; }
        } else {
            let mut msgv = None;

            // Search for alias
            let mut alias_cmd = None;
            {
                let state = state.lock().unwrap();
                if let Some(db) = &state.db {
                    alias_cmd = get_alias(&db, &cmd);
                }
            }

            // Search for command and exec
            if let Some(c) = self.commands.get(&cmd.as_str()) {
                if context.auth >= c.auth { msgv = c.exec(state, &context, args); }
            }
            // Else search for alias and exec
            else if let Some(alias_cmd) = alias_cmd {
                let (c, args) = pop_cmd(alias_cmd.clone());
                if let Some(c) = self.commands.get(c.as_str()) {
                    if context.auth >= c.auth { msgv = c.exec(state, &context, args); }
                }
            }
            return msgv;
        }
    }


}

pub struct Cmd {
    func: fn(t_state: Arc<Mutex<ThreadState>>, context: &Context, Option<String>) -> Option<Vec<String>>,
    pub bucket: Option<Bucket>,
    pub auth: Auth,
}

impl Cmd {
    pub fn exec(&self, t_state: Arc<Mutex<ThreadState>>, context: &Context, args: Option<String>) -> Option<Vec<String>> {
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

fn say() -> Cmd {
    Cmd {
        func: |_, _, args| {
            if let Some(args) = args {
                Some(vec!(args))
            } else { None }
        },
        bucket: None,
        auth: Auth::Mod,
    }
}

fn count() -> Cmd {
    Cmd {
        func: |_, _, args| {
            if let Some(args) = args {
                match args.parse::<u32>() {
                    Ok(u) => {
                        let mut v = Vec::new();
                        for i in 0..u {
                            v.push(format!("{}", i));
                        }
                        Some(v)
                    },
                    Err(_) => None,
                }
            } else { None }
        },
        bucket: None,
        auth: Auth::Owner,
    }
}

fn thicc() -> Cmd {
    Cmd {
        func: |_, _, args| {
            if let Some(arg) = args {
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
                Some(vec!(response))
            } else { None }
        },
        bucket: None,
        auth: Auth::Viewer,
    }
}

fn eightball() -> Cmd {
    Cmd {
        func: |_, _, _| {
            let mut rng = thread_rng();
            let answers = vec!("It is certain.",
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
                               "Very doubtful.");
            Some(vec!(String::from(answers[rng.gen_range(0,answers.len())])))
        },
        bucket: None,
        auth: Auth::Viewer,
    }
}

fn coinflip() -> Cmd {
    Cmd {
        func: |_, _, args| {
            if let Some(args) = args {
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
                    return Some(vec!(r));
                } else {
                    return None
                }
            } else {
                if random() {
                    return Some(vec!(String::from("Heads")))
                } else {
                    return Some(vec!(String::from("Tails")))
                }
            }
        },
        bucket: None,
        auth: Auth::Viewer,
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
                            let between = Uniform::from(1..(num_die_faces+1));
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
                Some(vec!(format!("{}", sum)))
            } else {
                let roll = rng.gen_range(1,20);
                let mut roll_string = format!("{}", roll);
                if roll_string == "20" {
                    roll_string.push_str(" PogChamp");
                } else if roll_string == "1" {
                    roll_string.push_str(" NotLikeThis");
                }
                Some(vec!(roll_string))
            }
        },
        bucket: None,
        auth: Auth::Viewer,
    }
}

fn quote() -> Cmd {
    Cmd {
        func: |t_state, _, _| {
            let t_state = t_state.lock().unwrap();
            if let Some(db) = &t_state.db {
                let quote = db.query_row("SELECT * FROM quote ORDER BY RANDOM() LIMIT 1;", &[], |row| {
                    let id: u32 = row.get(0);
                    let q: String = row.get(1);
                    format!("[{}] {}", id, q)
                });
                if let Ok(quote) = quote {
                    let mut v = Vec::new();
                    v.push(quote);
                    return Some(v)
                }
            }
            None
        },
        bucket: None,
        auth: Auth::Viewer,
    }
}

fn quoteadd() -> Cmd {
    Cmd {
        func: |t_state, _, args| {
            if let Some(args) = args {
                let t_state = t_state.lock().unwrap();
                if let Some(db) = &t_state.db {
                    db.execute("INSERT INTO quote (quote) values (?1)",
                    &[&args]).unwrap();
                }
            }
            None
        },
        bucket: None,
        auth: Auth::Mod,
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
        auth: Auth::Mod,
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
        auth: Auth::Owner,
    }
}

fn version() -> Cmd {
    Cmd {
        func: |_, _, _| {
            let v = env!("GIT_VERSION");
            Some(vec!(String::from(v)))
        },
        bucket: None,
        auth: Auth::Owner,
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
//                                        Helper Functions                                        //
////////////////////////////////////////////////////////////////////////////////////////////////////


fn pop_cmd(s: String) -> (String, Option<String>) {
    let s = String::from(s.trim_left());
    let argv: Vec<&str> = s.splitn(2, " ").collect();

    if argv.len() == 2 {
        (argv[0].to_string(), Some(argv[1].trim().to_string()))
    } else {
        (argv[0].to_string(), None)
    }
}

fn rm_alias(db: &Connection, alias: &str) {
    let _ = db.execute("DELETE FROM alias WHERE alias=?1", &[&alias]);
}

fn add_alias(db: &Connection, alias: &str, cmd: &str) {
    let _ = db.execute("INSERT INTO alias (alias, command) VALUES (?1, ?2)", &[&alias, &cmd]);
}

fn get_alias(db: &Connection, alias: &str) -> Option<String> {
    let cmd = db.query_row("SELECT * FROM alias WHERE alias=?1", &[&alias], |row| {
        row.get(2)
    });
    if let Ok(cmd) = cmd {
        Some(cmd)
    } else {
        None
    }
}
