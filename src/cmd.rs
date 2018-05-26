use std::time::{Duration, Instant};
use std::collections::{HashMap, LinkedList};

pub enum Auth {
    Owner,
    Streamer,
    Mod,
    Subscriber,
    Viewer
}

pub struct Bucket {
    pub count: u32,
    pub interval: Duration,
}

pub struct CmdList {
    commands: HashMap<String, Cmd>,
    aliases: HashMap<String, (String, String)>,
    log: LinkedList<(String, Instant)>
}

impl CmdList {
    fn new() -> Self {
        let mut commands = HashMap::new();
        let mut aliases = HashMap::new();
        let log = LinkedList::new();

        commands.insert("say", say());

        Self {
            commands,
            aliases,
            log,
        }
    }

    fn exec(mut self, cmd: &str, args: &str) {
        if cmd == "alias" {
            // FIXME: Assumes args will be split into 3 arguments (no error handling)
            let args = args.to_string();
            let argv: Vec<&str> = args.splitn(3, " ").collect();
            self.aliases.insert(argv[0].to_string(), (argv[1].to_string(), argv[2].to_string()));
        } else {
            // Search for command and exec
            if let Some(c) = self.commands.get(cmd) {
                c.exec(args);
                return
            }
            // Else search for alias and exec
            if let Some((c, args)) = self.aliases.get(cmd) {
                if let Some(c) = self.commands.get(c) {
                    c.exec(args);
                }
            }
        }
    }
}

pub struct Cmd {
    func: fn(&str),
    pub bucket: Option<Bucket>,
    pub auth: Auth,
}

impl Cmd {
    pub fn exec(&self, args: &str) {
        (self.func)(args);
    }
}

pub fn say() -> Cmd {
    Cmd {
        func: |args| {
            println!("{}", args);
        },
        bucket: None,
        auth: Auth::Owner,
    }
}
