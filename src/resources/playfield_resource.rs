use std::ops::Index;
use std::ops::IndexMut;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Playfields {
    pub keys: Vec<PlayfieldResource>,
}

impl Default for Playfields {
    fn default() -> Self {
        Playfields {
            keys: vec![PlayfieldResource::default()],
        }
    }
}

impl Playfields {
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

// index easier so you dont need access keys all the time
impl Index<usize> for Playfields {
    type Output = PlayfieldResource;

    fn index(&self, i: usize) -> &PlayfieldResource {
        &self.keys[i]
    }
}

// index mutably easier so you dont need access keys all the time
impl IndexMut<usize> for Playfields {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut PlayfieldResource {
        &mut self.keys[i]
    }
}
