pub struct ClientConfig {
    pub frame_cap: u32,
    pub vsync: bool
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self { 
            frame_cap: 60, 
            vsync: true 
        }
    }
}

