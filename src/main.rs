#![allow(unused_imports)]
#![allow(dead_code)]

mod engine;
mod helpers;
mod scripts;

fn main() {
	engine::run(500., 500., "everpuzzle")
}