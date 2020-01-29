use crate::engine::App;
use crate::scripts::*;
use crate::helpers::*;
use GarbageStates::*;

#[derive(Copy, Clone)]
pub enum GarbageStates {
	Idle,
	Hang,
	Clear,
}

pub struct Garbage {
	pub hframe: f32,
    pub vframe: f32,
    pub state: GarbageStates,
    pub offset: V2,
}

impl Default for Garbage {
    fn default() -> Self {
        Self {
            hframe: 0.,
            vframe: 8.,
            state: Idle,
            offset: V2::zero(),
        }
    }
}


