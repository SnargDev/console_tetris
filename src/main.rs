use ndarray::prelude::*;
use ndarray::Array;
use colored::Colorize;

//https://tetris.wiki/Tetris_Guideline
//https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
//https://stackoverflow.com/questions/233850/tetris-piece-rotation-algorithm

fn main() {

    let p = Piece::new(Block::Green);
    println!("{:?}", p.matrix);

    /*let x = 10;
    let y = 40;
    let mut a = Array::<Block, _>::from_elem((y, x).f(), Block::Red);
    a[[0,5]] = Block::LightBlue;

    let mut idx = 0;
    for b in Block::VALUES{
        a[[0,idx]] = b;
        idx += 1;
    }

    for (i, b) in a.iter().enumerate(){
        b.print();
        if (i+1) % 10 == 0{
            println!("\n");
        }
    }*/
    
    //println!("{:?}", a);

    //let b = Block::Green;
    //println!("Green: {}", b as i32);
    //println!("Red: {}", Block::Red as i32);
    //println!("None: {}", Block::None as i32);

    //https://github.com/rust-ndarray/ndarray/blob/master/README-quick-start.md
}

#[derive(Copy, Clone, Debug)]
pub enum Block {
    LightBlue,
    DarkBlue,
    Orange,
    Yellow,
    Green,
    Red,
    Magenta,

    None
}

impl Block{
    const VALUES: [Self; 8] = [Block::LightBlue, Block::DarkBlue, Block::Orange, Block::Yellow, Block::Green, Block::Red, Block::Magenta, Block::None];
}

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);


impl Block{
    fn print(&self){

        /*
        print!("{}",
        
        match self{
            Block::LightBlue => Colorize::bright_blue("[]"),
            Block::DarkBlue => Colorize::blue("[]"),
            Block::Orange => Colorize::truecolor("[]", 128, 128, 0),

            _ => Colorize::black("  ")
        });*/

        let colors = [
            Color(50,50,180),
            Color(0,0,255),
            Color(128, 30, 0),
            Color(128, 128, 0),
            Color(0,255,0),
            Color(255, 0, 0),
            Color(128, 0, 128),
            Color(0,0,0)
        ];

        let c = colors[*self as usize];
        print!("{}", Colorize::truecolor("[]", c.0, c.1, c.2));
    }

}


pub struct Piece{
    matrix: Array2<Block>
}

impl Piece{
    pub fn new(block_type: Block) -> Piece{

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

        Piece {matrix}
    }
}

/*
1. Rendering
-2d matrix to store color
-iterate over color matrix and render
 */