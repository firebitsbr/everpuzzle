use amethyst::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayfieldResource {
    pub x: f32,
    pub y: f32,
    pub level: usize,
}

impl Default for PlayfieldResource {
    fn default() -> Self {
        PlayfieldResource {
            x: 0.0,
            y: 0.0,
            level: 0,
        }
    }
}
