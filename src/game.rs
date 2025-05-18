use crate::Block;
use crate::Piece;
use ndarray::prelude::*;
use ndarray::Array;
use std::time::{Duration, Instant};
use std::thread::sleep;


    pub fn run(){
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

            //i want to revert this but cant
            for (i, b) in a.iter().enumerate(){
                
                let x = i % size_x;
                let y = (i - x)/size_x;

                if y+1 < size_y{
                    new_screen[[y+1, x]] = *b;
                }
            }

            a = new_screen;


            //this seemingly just fills up the screen with invisible chars, which is good enough i guess but i dont like it
            print!("{}[2J", 27 as char);


            //print screen
            let mut out = String::from("");
            for (i, b) in a.iter().enumerate(){
                out += &b.get_string_rep();
                if (i+1) % 10 == 0{
                    out += "\n";
                }
            }
            println!("{}", out);


            let time = Instant::now();
            if let Some(sleep_for) = frame_time.checked_sub(time.duration_since(last_frame)){

                if !sleep_for.is_zero() {
                    sleep(sleep_for);
                }
            }

            last_frame = Instant::now();
        }
    }
