#[derive(Debug, Serialize, Deserialize)]
pub struct PlayfieldResource {
    pub x: f32,
    pub y: f32,
    pub start_level: usize, // set internally to the beginning set level
    pub level: usize,       // will change over time, should be reset to start
}

impl Default for PlayfieldResource {
    fn default() -> Self {
        PlayfieldResource {
            x: 0.0,
            y: 0.0,
            start_level: 0,
            level: 0,
        }
    }
}
