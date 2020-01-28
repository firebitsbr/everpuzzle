#![allow(unused_imports)]
#![allow(dead_code)]
// TODO(Skytrias): remove allows on shipping release, i prefer this so i dont get bombarded after small tweaks

mod engine;
mod helpers;
mod scripts;

fn main() {
    engine::run(500., 500., "everpuzzle")
}
