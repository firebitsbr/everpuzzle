use crate::helpers::{GRID_HEIGHT, GRID_TOTAL, GRID_WIDTH};

// flow of xyi increase, i usually is converted automatically
enum IteratorFlow {
    XY,        // usual i 0..GRID_TOTAL
    YXReverse, // y goes GRID_HEGIHT..0 THEN x GRID_WIDTH..0
}

// often times you always want x, y, and i so here is an iterator that gives all of them
struct BoundIterator {
    x: usize,
    y: usize,
    i: usize,
    steps: usize, // detect how many steps its in
    flow: IteratorFlow,
}

impl Default for BoundIterator {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            i: 0,
            steps: 0,
            flow: IteratorFlow::XY,
        }
    }
}

impl Iterator for BoundIterator {
    type Item = (usize, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.flow {
            IteratorFlow::XY => {
                if self.steps == GRID_TOTAL {
                    return None;
                }

                self.steps += 1;

                let i = self.steps - 1;
                let x = self.x;
                let y = self.y;

                if self.x < GRID_WIDTH - 1 {
                    self.x += 1;
                } else {
                    self.y += 1;
                    self.x = 0;
                }

                Some((x, y, i))
            }

            IteratorFlow::YXReverse => {
                if self.steps == GRID_TOTAL {
                    //println!("------------------------------- stopped");
                    return None;
                }

                self.steps += 1;

                let x = self.x;
                let y = self.y;
                let i = y * GRID_WIDTH + x;

                if self.y != 0 {
                    self.y -= 1;
                //println!("c {}    x {}    y {}    i {}", self.steps - 1, x, y, i);
                } else {
                    //println!("c {}    x {}    y {}    i {}", self.steps - 1, x, y, i);
                    //println!("--------------------");

                    // let last self.y != 0 go through
                    if self.x != 0 {
                        self.x -= 1;
                    }

                    self.y = GRID_HEIGHT - 1;
                }

                Some((x, y, i))
            }
        }
    }
}

pub fn iter_xy() -> impl Iterator<Item = (usize, usize, usize)> {
    BoundIterator::default()
}

pub fn iter_yx_rev() -> impl Iterator<Item = (usize, usize, usize)> {
    BoundIterator {
        x: GRID_WIDTH - 1,
        y: GRID_HEIGHT - 1,
        flow: IteratorFlow::YXReverse,
        ..Default::default()
    }
}
