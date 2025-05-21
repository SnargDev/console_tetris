use crate::Block;
use crate::Piece;
use ndarray::prelude::*;
use ndarray::Array;
use std::time::{Duration, Instant};
use std::thread::sleep;

use crate::input;

pub fn run(rx: std::sync::mpsc::Receiver<input::InputPackage>){

    //option because there should be an update inbetween placing a piece and spawning the next one
    //because otherwise the player could maybe hard drop onto blocks that are being cleared that turn
    
    //println!("{:?}", piece.unwrap().matrix);

    let spawn_x = 5;
    let spawn_y = 10;

    let mut piece: Option<Piece> = None;//Some(Piece::new(Block::Red, spawn_x, spawn_y));

    let size_x = 10;
    let size_y = 40;
    let mut a = Array::<Block, _>::from_elem((size_y, size_x).f(), Block::None);
    
    /*a[[0,5]] = Block::LightBlue;

    let mut idx = 0;
    for b in Block::VALUES{
        a[[0,idx]] = b;
        idx += 1;
    }*/




    let frame_time = Duration::from_millis(250);
    let mut last_frame = Instant::now();

    loop {
        //gravity
        if let Some(ref mut p) = piece{

            //let mut vec = vec![];
            //for item in rx.try_iter(){
            //    vec.push(item);
            //}

            if let Some(package) = rx.try_iter().collect::<Vec<input::InputPackage>>().last(){
                p.x += package.move_x;
            }

            /*
            for s in rx.try_iter(){
                use input::InputData::*;
                match s {
                    Left => p.x -= 1,
                    Right => p.x += 1
                }
            }*/

            p.y += 1;


            let (matrix_x, _) = p.matrix.dim();
            //the code in the closure can be made better by using non-constant values here
            let max_x = size_x - 1;//matrix_x;
            let max_y = size_y - 1;//matrix_y;

            //TODO seperate movement and gravity
            //are all blocks inside the playing field?

            //could use iter().enumerate().any here but eh
            let dropped = !p.matrix.iter().enumerate().all(|(i, b)| *b == Block::None || {

                let x = i % matrix_x;
                let y = p.y + ((i - x)/matrix_x) as i16;
                let x = x as i16 + p.x;

                y <= max_y as i16 && x <= max_x as i16 && x > 0 && a[[y as usize, x as usize]] == Block::None
            });

            if dropped{
                println!("dropped piece");
                p.y -= 1;
            }

            //put piece on the matrix
            for (i, b) in p.matrix.iter().enumerate().filter(|(_, b)| **b != Block::None){
                let x = i % matrix_x;
                let y = (i - x)/matrix_x;

                a[[y + p.y as usize, x + p.x as usize]] = *b;
            }

            if dropped{
                piece = None;
            }
        }

        //print screen
        // *2 because every block takes up 2 chars
        let mut out = String::with_capacity(size_x * size_y * 2);
        for (i, b) in a.iter().enumerate(){
            out += &b.get_string_rep();
            if (i+1) % 10 == 0{
                out += "\n";
            }
        }

        //this seemingly just fills up the screen with invisible chars, which is good enough i guess but i dont like it
        print!("{}[2J", 27 as char);

        println!("{}", out);

        if let Some(ref p) = piece{

            let (sy, sx) = p.matrix.dim();

            for y in 0..sy{
                for x in 0..sx{
                    if p.matrix[[y,x]] != Block::None{

                        assert_ne!(a[[y + p.y as usize, x+ p.x as usize]], Block::None);
                        a[[y + p.y as usize, x + p.x as usize]] = Block::None;
                    } 
                }
            }
        }
        else {
            piece = Some(Piece::new(Block::VALUES[rand::random_range(0..(Block::VALUES.len()-1))], spawn_x, spawn_y));
        }

        let time = Instant::now();
        if let Some(sleep_for) = frame_time.checked_sub(time.duration_since(last_frame)){

            if !sleep_for.is_zero() {
                sleep(sleep_for);
            }
        }

        last_frame = Instant::now();
    }
}
