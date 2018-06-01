use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use rusqlite::Connection;

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
