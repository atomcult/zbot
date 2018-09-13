use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct MainState {
    pub shutdown: bool,
}

impl MainState {
    pub fn new() -> Arc<Mutex<Self>> {
        let state = MainState { shutdown: false };
        Arc::new(Mutex::new(state))
    }
}

pub struct ThreadState {
    pub main: Arc<Mutex<MainState>>,
    pub db: Option<Connection>,
    pub poll_id: Option<u32>,
}

impl ThreadState {
    pub fn new(main: Arc<Mutex<MainState>>) -> Arc<Mutex<ThreadState>> {
        let state = ThreadState { main, db: None, poll_id: None };
        Arc::new(Mutex::new(state))
    }
}
