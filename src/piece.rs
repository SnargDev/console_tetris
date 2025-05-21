use ndarray::prelude::*;
use ndarray::Array;
use crate::block::Block;

pub struct Piece{
    pub matrix: Array2<Block>,
    pub x: i16,
    //is never negative but should be i16 for consistency
    pub y: i16,
}

impl Piece{
    pub fn new(block_type: Block, x: i16, y: i16) -> Piece{

        let mut matrix = match block_type{
            Block::LightBlue => Array::<Block, _>::from_elem((4, 4).f(), Block::None),
            Block::None => panic!("Tried to create a piece of type None"),
            _ => Array::<Block, _>::from_elem((3, 3).f(), Block::None),
        };

        let fill = match block_type {
            Block::LightBlue => vec![(1,0), (1,1), (1,2), (1,3)],
            Block::DarkBlue => vec![(0,0), (1,0), (1,1), (1,2)],
            Block::Orange => vec![(2,0), (1,0), (1,1), (1,2)],
            Block::Yellow => vec![(0,1), (0,2), (1,1), (1,2)],
            Block::Green => vec![(0,1),(0,2),(1,0),(1,1)],
            Block::Magenta => vec![(0,1), (1,0), (1,1), (1,2)],
            Block::Red => vec![(0,0), (0,1), (1,1), (1,2)],
            
            Block::None => panic!("How even")
        };

        for (x,y) in fill{
            matrix[[x,y]] = block_type;
        }

        Piece {matrix, x, y}
    }
}