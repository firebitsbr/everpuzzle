#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

/// loads a file at runtime in debug mode, includes the file into the binary in release mode
macro_rules! load_file {
    ($path:expr) => {
        if cfg!(debug_assertions) {
            std::fs::read($path).expect("Failed to open file")
        } else {
            include_bytes!(concat!("../../", $path)).to_vec()
        }
    };
}

mod engine;
mod helpers;
mod scripts;

use miniquad::*;

/// starts the entire game
fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(engine::App::new(&mut ctx), ctx)
    });
}
