use crate::Block;
use crate::Piece;
use ndarray::prelude::*;
use ndarray::Array;
use std::time::{Duration, Instant};
use std::thread::sleep;

use crate::input::InputPackage;
use std::sync::{Arc, Mutex};

const RENDER_SIZE: usize = 4440;

pub fn run(package_access: Arc<Mutex<InputPackage>>){

    //option because there should be an update inbetween placing a piece and spawning the next one
    //because otherwise the player could maybe hard drop onto blocks that are being cleared that turn
    //also this lets me set it to none once dropped, making the rest of the loop a little simpler
    let mut piece: Option<Piece> = None;

    let spawn_x = 5;
    let spawn_y = 10;

    let size_x = 10;
    let size_y = 40;
    let mut field = Array::<Block, _>::from_elem((size_y, size_x).f(), Block::None);

    let frame_time = Duration::from_millis(250);
    let mut last_frame = Instant::now();

    loop {

        let package = 
        {
            let mut mutex = package_access.lock().unwrap();
            let new_package = mutex.clone();
            *mutex = InputPackage::new();

            new_package
        };

        

        //gravity
        if let Some(ref mut p) = piece{

            

            if package.move_x != 0{
                p.x += package.move_x;
                if !is_piece_valid(p, &field){
                    p.x -= package.move_x;
                }
            }

            p.y += 1;


            let (_, matrix_x) = p.matrix.dim();


            //could use iter().enumerate().any here but eh
            let dropped = !is_piece_valid(p, &field);

            if dropped{
                //println!("dropped piece");
                p.y -= 1;
            }

            //put piece on the matrix
            for (i, b) in p.matrix.iter().enumerate().filter(|(_, b)| **b != Block::None){
                let x = i % matrix_x;
                let y = (i - x)/matrix_x;

                field[[y + p.y as usize, x + p.x as usize]] = *b;
            }

            if dropped{
                piece = None;
            }
        }

        //print screen
        let mut out = String::with_capacity(4440);//i just checked the size once, should probably do this mathematically
        for (i, b) in field.iter().enumerate(){
            out += &b.get_string_rep();
            if (i+1) % 10 == 0{
                out += "\n";
            }
        }

        assert_eq!(out.len(), RENDER_SIZE);

        //println!("{}", out.len());
        //return;

        //this seemingly just fills up the screen with invisible chars, which is good enough i guess but i dont like it
        print!("{}[2J", 27 as char);

        println!("{}", out);

        if let Some(ref p) = piece{

            let (sy, sx) = p.matrix.dim();

            for y in 0..sy{
                for x in 0..sx{
                    if p.matrix[[y,x]] != Block::None{

                        assert_ne!(field[[y + p.y as usize, x+ p.x as usize]], Block::None);
                        field[[y + p.y as usize, x + p.x as usize]] = Block::None;
                    } 
                }
            }
        }
        else {
            piece = Some(Piece::new(Block::VALUES[rand::random_range(0..(Block::VALUES.len()-1))], spawn_x, spawn_y));
        }


        
        let time = Instant::now();
        debug_assert!(Duration::from_millis(2) > time.duration_since(last_frame), "{}", time.duration_since(last_frame).as_millis());
        if let Some(sleep_for) = frame_time.checked_sub(time.duration_since(last_frame)){

            if !sleep_for.is_zero() {
                sleep(sleep_for);
            }
        }

        last_frame = Instant::now();
    }
}

use ndarray::OwnedRepr;
fn is_piece_valid(piece: &Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> bool{
    let (_, matrix_x) = piece.matrix.dim();


    let(my, mx) = field.dim();
    let max_x = mx-1;
    let max_y = my-1;

    piece.matrix.iter().enumerate().all(|(i, b)| *b == Block::None || {

    let tmp = i % matrix_x;
    let y = piece.y + ((i - tmp)/matrix_x) as i16;
    let x = tmp as i16 + piece.x;

    y <= max_y as i16 && x <= max_x as i16 && x > 0 && field[[y as usize, x as usize]] == Block::None
    })
}
