pub enum Auth {
    Owner,
    Streamer,
    Mod,
    Subscriber,
    Viewer
}

impl Auth {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => Auth::Owner,
            "streamer" => Auth::Streamer,
            "mod" => Auth::Mod,
            "subscriber" => Auth::Subscriber,
            _ => Auth::Viewer,
        }
    }
}
