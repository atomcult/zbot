bitflags! {
    pub struct Permissions: u8 {
        const ReadOnly = 0b10000000;
        const Owner    = 0b00010000;
        const Streamer = 0b00001000;
        const Mod      = 0b00000100;
        const Sub      = 0b00000010;
        const Viewer   = 0b00000001;
    }
}
