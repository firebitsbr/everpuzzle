#![allow(dead_code)]
use crate::data::playfield_data::{BLOCKS, COLUMNS};
use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};
use std::ops::Index;

pub struct Stack {
    p_id: usize,
    block_entities: Vec<Entity>,
    pub cursor_entity: Entity,
}

impl Stack {
    pub fn new(p_id: usize, block_entities: Vec<Entity>, cursor_entity: Entity) -> Stack {
        Stack {
            p_id,
            block_entities,
            cursor_entity,
        }
    }

    // convert an x and y coordinate to i
    // use this if you want to back convert from an x and y
    // this is most often used when only one parameter changes and the other one stays
    // example: for x in 0..10 {
    // 		xy2i(x, 0) // searches through 0 until 10 from y at 0
    // }
    pub fn coordinates_to_index(x: usize, y: usize) -> usize {
        y * COLUMNS + x
    }

    // use this instead of calling from_xy multiple times
    // converts an iterator i back to x and y
    pub fn index_to_coordinates(index: usize) -> (usize, usize) {
        (index % COLUMNS, index / COLUMNS)
    }

    // returns an iterator of entity references
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.block_entities.iter()
    }
}

// impls index for stack so that you can directly access the block_entities
// by calling stack[usize]
impl Index<usize> for Stack {
    type Output = Entity;

    fn index(&self, i: usize) -> &Entity {
        &self.block_entities[i + &self.p_id * BLOCKS]
    }
}

// impls index for stack so that you can directly access the block_entities
// by calling stack[(usize, usize)]
impl Index<(usize, usize)> for Stack {
    type Output = Entity;

    fn index(&self, (x, y): (usize, usize)) -> &Entity {
        &self.block_entities[Stack::coordinates_to_index(x, y) + &self.p_id * BLOCKS]
    }
}

impl Component for Stack {
    type Storage = DenseVecStorage<Self>;
}
