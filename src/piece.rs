use crate::block::Block;
use ndarray::Array;
use ndarray::prelude::*;

use crate::input::Rotation;
use ndarray::OwnedRepr;

pub struct Piece {
    pub matrix: Array2<Block>,
    pub x: i16,
    pub y: i16,
}

impl Piece {
    pub fn new(block_type: Block, x: i16, y: i16) -> Piece {
        let mut matrix = match block_type {
            Block::LightBlue => Array::<Block, _>::from_elem((4, 4).f(), Block::Void),
            Block::Yellow => {
                return Piece {
                    matrix: Array::<Block, _>::from_elem((2, 2).f(), Block::Yellow),
                    x,
                    y,
                };
            }
            Block::Void => panic!("Tried to create a piece of type Void"),
            _ => Array::<Block, _>::from_elem((3, 3).f(), Block::Void),
        };

        let fill = match block_type {
            Block::LightBlue => vec![(1, 0), (1, 1), (1, 2), (1, 3)],
            Block::DarkBlue => vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            Block::Orange => vec![(0, 2), (1, 0), (1, 1), (1, 2)],
            Block::Yellow => vec![],
            Block::Green => vec![(0, 1), (0, 2), (1, 0), (1, 1)],
            Block::Magenta => vec![(0, 1), (1, 0), (1, 1), (1, 2)],
            Block::Red => vec![(0, 0), (0, 1), (1, 1), (1, 2)],

            Block::Void => panic!("How even"),
        };

        for (x, y) in fill {
            matrix[[x, y]] = block_type;
        }

        Piece { matrix, x, y }
    }

    pub fn matrix_rotated(
        &self,
        direction: Rotation,
    ) -> ArrayBase<ndarray::OwnedRepr<Block>, Dim<[usize; 2]>> {
        //for x/y mult
        let (xm, ym) = if direction == Rotation::Clockwise {
            (1, -1)
        } else {
            (-1, 1)
        };

        let (my, mx) = self.matrix.dim();
        let my = my as isize;
        let mx = mx as isize;
        assert_eq!(my, mx);

        //i decided to copy this for slightly better performance and also because im lazy and dont want to deal with int rounding
        match my {
            3 => {
                let mut rotated = Array::<Block, _>::from_elem(self.matrix.dim().f(), Block::Void);

                for (i, b) in self.matrix.iter().enumerate() {
                    let i = i as isize;
                    let matrix_x = self.matrix.dim().1 as isize;

                    let tmp: isize = i % matrix_x;
                    let x: isize = tmp - 1;
                    let y: isize = (i - tmp) / matrix_x - 1;

                    rotated[[
                        (x * xm + 1)
                            .try_into()
                            .expect("should have received a valid usize"),
                        (y * ym + 1)
                            .try_into()
                            .expect("should have received a valid usize"),
                    ]] = *b;
                }

                rotated
            }

            //just the I block
            4 => {
                let mut rotated = Array::<Block, _>::from_elem(self.matrix.dim().f(), Block::Void);

                for (i, b) in self.matrix.iter().enumerate() {
                    let i = i as isize;

                    let tmp: isize = i % mx;

                    //0 and 1 negative, else 1 and 2
                    //0 1 < 2 => -2, -1
                    //2 3 >= 2 => 1 2
                    let x: isize = if tmp < 2 { tmp - 2 } else { tmp - 1 };
                    debug_assert!([-2, -1, 1, 2].contains(&x), "x was {x}");

                    let y: isize = (i - tmp) / mx;
                    let y: isize = if y < 2 { y - 2 } else { y - 1 };
                    debug_assert!([-2, -1, 1, 2].contains(&y), "y was {y}");

                    //swap
                    let nx = y * ym;
                    let ny = x * xm;

                    let nx = if nx < 0 { nx + 2 } else { nx + 1 };
                    let ny = if ny < 0 { ny + 2 } else { ny + 1 };

                    rotated[[
                        ny.try_into().expect("should have received a valid usize"),
                        nx.try_into().expect("should have received a valid usize"),
                    ]] = *b;
                }

                rotated
            }

            //yellow block doesnt need to be changed
            2 => self.matrix.clone(),

            _ => panic!(
                "matrix_rotated only implements operations for 3x3, 4x4 and the yellow block"
            ),
        }
    }

    pub fn add_to_field(&self, field: &mut ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) {
        let matrix_x = self.matrix.dim().1;

        for (i, b) in self
            .matrix
            .iter()
            .enumerate()
            .filter(|(_, b)| **b != Block::Void)
        {
            let x = i % matrix_x;
            let y = (i - x) / matrix_x;

            field[[(y as i16 + self.y) as usize, (x as i16 + self.x) as usize]] = *b;
        }
    }

    pub fn remove_from_field(&self, field: &mut ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) {
        let (sy, sx) = self.matrix.dim();

        for y in 0..sy {
            for x in 0..sx {
                if self.matrix[[y, x]] != Block::Void {
                    assert_ne!(
                        field[[(y as i16 + self.y) as usize, (x as i16 + self.x) as usize]],
                        Block::Void
                    );
                    field[[(y as i16 + self.y) as usize, (x as i16 + self.x) as usize]] =
                        Block::Void;
                }
            }
        }
    }

    fn get_block_validation_closure<'a>(
        &'a self,
        field: &'a ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    ) -> Box<dyn Fn((usize, &Block)) -> bool + 'a> {
        let (_, matrix_x) = self.matrix.dim();

        let (my, mx) = field.dim();
        let max_x = mx - 1;
        let max_y = my - 1;

        Box::new(move |(i, b)| {
            *b == Block::Void || {
                let tmp = i % matrix_x;
                let y = self.y + ((i - tmp) / matrix_x) as i16;
                let x = tmp as i16 + self.x;

                //yes, the y >= 0 check is necessary. i do not know why. most confusing thing is that this is only relevant
                //when the block reaches the bottom of the sreen so y should be like 39. dont care enough to think about it cause its working now.
                y <= max_y as i16
                    && x <= max_x as i16
                    && x >= 0
                    && y >= 0
                    && field[[y as usize, x as usize]] == Block::Void
            }
        })
    }

    fn fail_count(&self, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> usize {
        //mediator closure to call the validator because filter requires the arguments to be a reference
        //didnt come up with this myself, rust people on discord are pretty cool
        let validation_closure = Self::get_block_validation_closure(self, field);
        self.matrix
            .iter()
            .enumerate()
            .filter(|(i, b)| !validation_closure((*i, b)))
            .count()
    }

    //this short curcuits as to not view the entire matrix every time
    fn fits_fail_limit(
        &self,
        field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
        limit: usize,
    ) -> (bool, usize) {
        let mut fails = 0;
        let validation_closure = self.get_block_validation_closure(field);
        for (i, b) in self.matrix.iter().enumerate() {
            if !validation_closure((i, b)) {
                fails += 1;
                if fails >= limit {
                    return (false, fails);
                }
            }
        }

        (true, fails)
    }

    pub fn perform_rotation(
        &mut self,
        field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
        rotate: Rotation,
    ) -> bool {
        if rotate == Rotation::Not {
            return false;
        }

        let rotated = self.matrix_rotated(rotate);
        //ideally i'd do this without clone, i think std::mem::swap/take could work
        let old_matrix = self.matrix.clone();
        self.matrix = rotated;

        let init_fails = self.fail_count(&field);
        let old_x = self.x;
        let old_y = self.y;

        //idk if these wall kicks are 100% accurate but im not going to read through half the tetris wiki and hardcode all the values for every piece, i like this
        if init_fails > 0 {
            //while fail count is decreasing
            //  move block
            // if fail count == 0 => done else back to initial position
            //if the only condition is for the fail count to <= prev then a piece could force itself through other blocks

            //up, down, right, left
            let adds = [(0, -1), (0, 1), (1, 0), (-1, 0)];

            let mut fails = init_fails;

            for add in adds {
                while fails > 0 {
                    self.move_by(add.0, add.1);

                    let (ok, f) = self.fits_fail_limit(&field, fails);
                    //apparently you could just do let (ok, fails) = ... but that doesnt work here? like it just shadows fails
                    fails = f;

                    if !ok {
                        self.x = old_x;
                        self.y = old_y;
                        break;
                    }
                }
            }
        }

        if !self.is_valid(&field) {
            self.matrix = old_matrix;
            return false;
        }

        return true;
    }

    pub fn is_valid(&self, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> bool {
        self.matrix
            .iter()
            .enumerate()
            .all(self.get_block_validation_closure(field))
    }

    pub fn move_by(&mut self, x: i16, y: i16) {
        self.x += x;
        self.y += y;
    }

    pub fn get_span(&self) -> (u8, u8) {
        (
            self.y as u8,
            (self.y as u8 + self.matrix.dim().0 as u8) as u8,
        )
    }

    pub fn get_row_string(&self, y: u8) -> String {
        let size = self.matrix.dim().0 as u8;
        if y >= size {
            Block::Void.get_string_rep_colored().repeat(4)
        } else {
            let mut s: String = "".to_string();
            for x in 0..size {
                s.push_str(&mut self.matrix[[y as usize, x as usize]].get_string_rep_colored());
            }

            if size < 4 {
                s.push_str(
                    &Block::Void
                        .get_string_rep_colored()
                        .repeat((4 - size).into()),
                );
            }

            s
        }
    }
}
