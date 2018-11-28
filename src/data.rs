pub mod block_data {
    pub const ROWS: usize = 12;
    pub const COLS: usize = 6;
    pub const BLOCKS: usize = ROWS * COLS;
}

// MOST OF THIS DATA CAN BE FOUND ON GITHUB @ PANELATTACK
// EVERY TIME WAS FRAME COUNTED OR ESTIMATED

pub mod playfield_data {
    // columns of the grid 6 vertically
    pub const COLS: usize = 6;
    // rows that are off the screen (used for garbage)
    pub const ROWS_INV: usize = 12;
    // rows that can be seen 12 horizontallyk
    pub const ROWS_VIS: usize = 12;
    // sum of rows
    pub const ROWS: usize = ROWS_INV + ROWS_VIS;
    // overall amount of blocks that will exist per stack
    pub const BLOCKS: usize = COLS * ROWS;
}

// texture size infos
pub mod block_sprite {
    // size of the block in pixels
    pub const BLOCK_SIZE: usize = 16;
    // TODO: Access multiple arrays better
    // pub const BLOCK_COLORS: [usize; [usize; ]] = [[0, 1, 2, 3, 4], [0, 1, 2, 3, 4, 5]]
    // the amount of colors that are availabe with each difficulty
    pub const NUMBER_COLORS_VS: [usize; 10] = [0,  0,  0,  0,  0,  0,  0,  0,  1,  1];
}

// block time frames per level 1 to 10
pub mod block_animation {
    // time the block will stay in air before fallign
    pub const HOVER_TIME: [usize; 10] = [12, 12, 11, 10, 9, 6, 5, 4, 3, 6];
    // time that each clearing pop will take
    pub const POP_TIME: [usize; 10] = [9, 9, 8, 8, 8, 8, 8, 7, 7, 7];
    // time that it takes to animate the flashing in clears
    pub const FLASH_TIME: [usize; 10] = [44, 44, 42, 42, 38, 36, 34, 32, 30, 28];
    // time the face of a block will be showing up for
    pub const FACE_TIME: [usize; 10] = [15, 14, 14, 13, 12, 11, 10, 10, 9, 8];

    // statblock time constants

    // time a swap will take
    pub const SWAP_TIME: usize = 5;
    // time the land will animate for
    pub const LAND_TIME: usize = 10;
}
