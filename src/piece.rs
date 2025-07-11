use crate::block::Block;
use ndarray::Array;
use ndarray::prelude::*;

use crate::input::Rotation;

pub struct Piece {
    pub matrix: Array2<Block>,
    pub x: i16,
    pub y: i16,
}

impl Piece {
    pub fn new(block_type: Block, x: i16, y: i16) -> Piece {
        let mut matrix = match block_type {
            Block::LightBlue => Array::<Block, _>::from_elem((4, 4).f(), Block::None),
            Block::Yellow => {
                return Piece {
                    matrix: Array::<Block, _>::from_elem((2, 2).f(), Block::Yellow),
                    x,
                    y,
                };
            }
            Block::None => panic!("Tried to create a piece of type None"),
            _ => Array::<Block, _>::from_elem((3, 3).f(), Block::None),
        };

        let fill = match block_type {
            Block::LightBlue => vec![(1, 0), (1, 1), (1, 2), (1, 3)],
            Block::DarkBlue => vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            Block::Orange => vec![(0, 2), (1, 0), (1, 1), (1, 2)],
            Block::Yellow => vec![],
            Block::Green => vec![(0, 1), (0, 2), (1, 0), (1, 1)],
            Block::Magenta => vec![(0, 1), (1, 0), (1, 1), (1, 2)],
            Block::Red => vec![(0, 0), (0, 1), (1, 1), (1, 2)],

            Block::None => panic!("How even"),
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
                let mut rotated = Array::<Block, _>::from_elem(self.matrix.dim().f(), Block::None);

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
                let mut rotated = Array::<Block, _>::from_elem(self.matrix.dim().f(), Block::None);

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
}

impl std::ops::Add<(i16, i16)> for Piece {
    type Output = Piece;
    fn add(self, rhs: (i16, i16)) -> Self::Output {
        Piece {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
            ..self
        }
    }
}

impl std::ops::AddAssign<(i16, i16)> for Piece {
    fn add_assign(&mut self, rhs: (i16, i16)) {
        self.x += rhs.0;
        self.y += rhs.1;
    }
}

impl std::ops::Sub<(i16, i16)> for Piece {
    type Output = Piece;
    fn sub(self, rhs: (i16, i16)) -> Self::Output {
        Piece {
            x: self.x - rhs.0,
            y: self.y - rhs.1,
            ..self
        }
    }
}

impl std::ops::SubAssign<(i16, i16)> for Piece {
    fn sub_assign(&mut self, rhs: (i16, i16)) {
        self.x -= rhs.0;
        self.y -= rhs.1;
    }
}
