use ndarray::prelude::*;
use ndarray::Array;
use colored::Colorize;
use std::time::{Duration, Instant};
use std::thread::sleep;

//https://tetris.wiki/Tetris_Guideline
//https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
//https://stackoverflow.com/questions/233850/tetris-piece-rotation-algorithm

fn main() {

    let p = Piece::new(Block::Green);
    println!("{:?}", p.matrix);

    let size_x = 10;
    let size_y = 40;
    let mut a = Array::<Block, _>::from_elem((size_y, size_x).f(), Block::Red);
    a[[0,5]] = Block::LightBlue;


    let mut idx = 0;
    for b in Block::VALUES{
        a[[0,idx]] = b;
        idx += 1;
    }




    let frame_time = Duration::from_millis(250);
    let mut last_frame = Instant::now();

    loop {
        
        

        let mut new_screen = Array::<Block, _>::from_elem((size_y, size_x).f(), Block::None);
        //println!("created new_screen. x: {}, y: {}", size_x, size_y);

        //i want to revert this but cant
        for (i, b) in a.iter().enumerate(){
            
            let x = i % size_x;
            let y = (i - x)/size_x;

            //println!("x: {}, y: {}", x, y);


            //new_screen[[y,x]] = Block::None;
            if y+1 < size_y{
                new_screen[[y+1, x]] = *b;
            }
        }

        a = new_screen;


        //println!("before clearing screen");
        //this seemingly just fills up the screen with invisible chars, which is good enough i guess but i dont like it
        print!("{}[2J", 27 as char);
        //println!("after clearing screen");


        //print all blocks
        let mut out = String::from("");
        for (i, b) in a.iter().enumerate(){
            //b.print();
            out += &b.get_string_rep();
            if (i+1) % 10 == 0{
                out += "\n";
                //println!("\n");
            }
        }
        println!("{}", out);


        let time = Instant::now();
        if let Some(sleep_for) = frame_time.checked_sub(time.duration_since(last_frame)){

            //println!("sleep for is {} ms", sleep_for.as_millis());
            if !sleep_for.is_zero() {
                sleep(sleep_for);
            }
        }

        last_frame = Instant::now();
    }

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

impl std::fmt::Display for Block{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);


impl Block{
    fn print(&self){

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

    fn get_string_rep(&self) -> String{
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
        format!("{}", Colorize::truecolor("[]", c.0, c.1, c.2))
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