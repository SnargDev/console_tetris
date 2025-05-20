use crate::Block;
use crate::Piece;
use ndarray::prelude::*;
use ndarray::Array;
use std::time::{Duration, Instant};
use std::thread::sleep;


    pub fn run(){

        //option because there should be an update inbetween placing a piece and spawning the next one
        //because otherwise the player could maybe hard drop onto blocks that are being cleared that turn
        let mut piece = Some(Piece::new(Block::Green, 5, 30));
        //println!("{:?}", piece.unwrap().matrix);

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
            /*
            let mut new_screen = Array::<Block, _>::from_elem((size_y, size_x).f(), Block::None);

            
            //i want to revert this but cant
            for (i, b) in a.iter().enumerate(){
                
                let x = i % size_x;
                let y = (i - x)/size_x;

                if y+1 < size_y{
                    new_screen[[y+1, x]] = *b;
                }
            }

            a = new_screen;*/


            //this seemingly just fills up the screen with invisible chars, which is good enough i guess but i dont like it
            print!("{}[2J", 27 as char);

            //gravity
            if let Some(ref mut p) = piece{
                println!("printing piece, y: {}", p.y);

                p.y += 1;

                //this whole part is stupid and needs to be fixed
                //the math is all wrong

                let (matrix_x, _) = p.matrix.dim();
                //the code in the closure can be made better by using non-constant values here
                let max_x = size_x - 1;//matrix_x;
                let max_y = size_y - 1;//matrix_y;

                //TODO seperate movement and gravity
                //are all blocks inside the playing field?

                //true -> render it that way
                //false -> decrement y, add piece to matrix


                //could use aniter().enumerate().any here but eh
                let dropped = !p.matrix.iter().enumerate().all(|(i, b)| *b == Block::None || {

                    let x = i % matrix_x;
                    let y = p.y + (i - x)/matrix_x;
                    let x = x + p.x;

                    y <= max_y && x <= max_x && a[[y,x]] == Block::None
                });

                if dropped{
                    println!("dropped piece");
                    p.y -= 1;
                }

                //put piece on the matrix
                for (i, b) in p.matrix.iter().enumerate().filter(|(_, b)| **b != Block::None){
                    let x = i % matrix_x;
                    let y = (i - x)/matrix_x;

                    a[[y + p.y, x + p.x]] = *b;
                }

                if dropped{
                    piece = None;
                }
            }

            //print screen
            let mut out = String::from("");
            for (i, b) in a.iter().enumerate(){
                out += &b.get_string_rep();
                if (i+1) % 10 == 0{
                    out += "\n";
                }
            }
            println!("{}", out);

            if let Some(ref p) = piece{

                let (sy, sx) = p.matrix.dim();

                for y in 0..sy{
                    for x in 0..sx{
                        if p.matrix[[y,x]] != Block::None{

                            assert_ne!(a[[y + p.y, x+ p.x]], Block::None);
                            a[[y + p.y, x + p.x]] = Block::None;
                        } 
                    }
                }
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
