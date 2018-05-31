use std::time::{Duration, Instant};
use std::collections::{HashMap, LinkedList};
use auth::Auth;


pub struct CmdList {
    commands: HashMap<&'static str, Cmd>,
    aliases: HashMap<String, String>,
    send_buffer: LinkedList<(String, Instant)>
}

impl CmdList {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        let aliases = HashMap::new();
        let send_buffer = LinkedList::new();

        commands.insert("say", say());

        Self {
            commands,
            aliases,
            send_buffer,
        }
    }

    pub fn exec(&mut self, command: &str) -> Option<Vec<String>> {
        let (cmd, args) = pop_cmd(command.to_string());
        if cmd == "alias" {
            if let Some(args) = args {
                let (alias, command) = pop_cmd(args);
                if let Some(command) = command {
                    // If alias is the same as a command name, do not add the alias
                    if let Some(_) = self.commands.get(alias.as_str()) {
                        return Some(vec!(format!("Cannot alias `{}`. Command with the same name exists already.", alias)))
                    }
                    self.aliases.insert(alias, command);
                    return None
                } else {
                    return match self.aliases.remove(&alias) {
                        Some(_) => None,
                        None => Some(vec!(format!("Alias `{}` could not be removed.", alias))),
                    }
                }
            } else {
                return Some(vec!(String::from("Usage: !alias <alias> <cmd> [args...]")))
            }
        } else {
            // Search for command and exec
            if let Some(c) = self.commands.get(&cmd.as_str()) {
                return c.exec(args)
            }
            // Else search for alias and exec
            if let Some(alias_cmd) = self.aliases.get(&cmd) {
                let (c, args) = pop_cmd(alias_cmd.clone());
                if let Some(c) = self.commands.get(c.as_str()) {
                    return c.exec(args)
                }
            }
        }
        None
    }
}

pub struct Cmd {
    func: fn(Option<String>) -> Option<Vec<String>>,
    pub bucket: Option<Bucket>,
    pub auth: Auth,
}

impl Cmd {
    pub fn exec(&self, args: Option<String>) -> Option<Vec<String>> {
        (self.func)(args)
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
        func: |args| {
            if let Some(args) = args {
                Some(vec!(args))
            } else { None }
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
