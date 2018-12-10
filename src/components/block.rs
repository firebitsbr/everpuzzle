#![allow(dead_code, unused_imports)]
use amethyst::ecs::prelude::{Component, DenseVecStorage};
use data::block_data::LAND_TIME;
use std::clone::Clone;
use std::marker::Copy;

#[derive(Debug, Copy, Clone)]
pub struct Block {
    pub kind: i32, // sprite_number or -1 meaning invisible
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub offset: (f32, f32),
    pub move_dir: f32,

    // fsm
    pub state: &'static str,
    pub counter: u32,

    // clear
    pub chainable: bool,
    pub clearing: bool,
    pub clear_counter: i32,
    pub clear_anim_counter: i32,
    pub clear_time: i32,
    pub clear_start_counter: i32,

    // animation
    pub anim_counter: u32,
    pub anim_offset: u32,

    // playfield data
    pub level: usize,
}

impl Default for Block {
    fn default() -> Block {
        Block {
            kind: -1,
            id: 0,
            x: 0,
            y: 0,
            offset: (0.0, 0.0),
            move_dir: 1.0,
            state: "IDLE",
            counter: 0,

            // clear variables
            chainable: false,
            clearing: false,
            clear_counter: 0,
            clear_anim_counter: 0,
            clear_time: 0,
            clear_start_counter: 0,

            // anim counters
            anim_counter: 0,
            anim_offset: 0,

            level: 0,
        }
    }
}

impl Block {
    pub fn new(id: u32, kind: i32, x: i32, y: i32, level: usize) -> Block {
        Block {
            id,
            kind,
            x,
            y,
            level,
            ..Default::default()
        }
    }

    // a block is empty when its kind is -1 so it turns invisible and
    // its state is always idle
    pub fn is_empty(&self) -> bool {
        self.kind == -1 && self.state == "IDLE"
    }

    // a block is swappable when:
    // its state isnt idle or its invisible,
    // other block isnt empty and currently in fall,
    // its state is land and its counter still below land time
    // valid blocks are currently swapping
    pub fn is_swappable(&self, other: &Block, above_block: Option<&Block>) -> bool {
        if let Some(above) = above_block {
            if above.state == "HANG" {
                return true;
            }
        }

        if !other.is_empty() && self.state == "FALL" {
            return true;
        }

        if self.state == "LAND" {
            return true;
        }

        if self.state == "IDLE" || self.kind == -1 {
            return true;
        }

        if other.kind != -1 && other.state == "MOVE" && self.state == "MOVE" {
            return true;
        }

        return false;
    }

    // a block is comboable under these conditions:
    // - its y is not at the bottom of the blocks - darkened column
    // - its kind is not invisible (-1)
    // - its state is Idle
    // - it's currently landing
    pub fn is_comboable(&self) -> bool {
        if self.y == 0 {
            return false;
        }

        // garbage

        if self.kind != -1 && self.state == "IDLE" {
            return true;
        }

        if self.state == "LAND" && self.counter < LAND_TIME {
            return true;
        }

        return false;
    }

    // whether this block is comboable with another given kind
    pub fn is_comboable_with(&self, other_kind: i32) -> bool {
        self.is_comboable() && other_kind != -1 && other_kind == self.kind
    }

    // set properties from another block
    // THIS SHOULD BE CHANGED WHENEVER DATA SHOULD PERSIST AFTER A FALL OR A SWAP!!!
    pub fn set_properties(&mut self, other: Block) {
        self.kind = other.kind;
        self.offset = other.offset;

        // fsm
        self.state = other.state;
        self.counter = other.counter;

        // clear
        self.chainable = other.chainable;
        self.clear_start_counter = other.clear_start_counter;

        // animation
        self.anim_counter = other.anim_counter;
        self.anim_offset = other.anim_offset;
    }

    // reset everything but the set variables that should remain
    // the same
    pub fn reset(&mut self) {
        *self = Block {
            id: self.id,
            x: self.x,
            y: self.y,
            level: self.level,
            ..Default::default()
        };
    }

    pub fn load(&mut self, other: Block) {
        *self = other;
    }
}

impl Component for Block {
    type Storage = DenseVecStorage<Self>;
}
