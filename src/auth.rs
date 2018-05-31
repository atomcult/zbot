use std::cmp::Ordering;

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

    fn as_u8(&self) -> u8 {
        match self {
            Auth::Owner      => 4,
            Auth::Streamer   => 3,
            Auth::Mod        => 2,
            Auth::Subscriber => 1,
            _                => 0,
        }
    }
}

impl Ord for Auth {
    fn cmp(&self, other: &Auth) -> Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

impl Eq for Auth {}

impl PartialOrd for Auth {
    fn partial_cmp(&self, other: &Auth) -> Option<Ordering> {
        Some(self.as_u8().cmp(&other.as_u8()))
    }
}

impl PartialEq for Auth {
    fn eq(&self, other: &Auth) -> bool {
        self.as_u8() == other.as_u8()
    }
}
