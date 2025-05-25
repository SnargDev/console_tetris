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

    let spawn_x = 5;
    let spawn_y = 10;

    //option because there should be an update inbetween placing a piece and spawning the next one
    //because otherwise the player could maybe hard drop onto blocks that are being cleared that turn
    //also this lets me set it to none once dropped, making the rest of the loop a little simpler
    let mut piece: Option<Piece> = Some(Piece::new(Block::Red, spawn_x, spawn_y));//None;

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


            use crate::input::Rotation;
            if package.rotate != Rotation::Not{
                let rotated = p.matrix_rotated(package.rotate);
                //ideally i'd do this without clone, i think std::mem::swap/take could work
                let old_matrix = p.matrix.clone();
                p.matrix = rotated;

                let init_fails = fail_count(p, &field);
                let old_x = p.x;
                let old_y = p.y;

                //idk if these wall kicks are 100% accurate but im not going to read through half the tetris wiki and hardcode all the values for every piece, i like this
                if init_fails > 0{

                    //while fail count is decreasing
                    //  move block
                    // if fail count == 0 => done else back to initial position
                    //if the only condition is for the fail count to <= prev then a piece could force itself through other blocks

                    //up, down, right, left
                    let adds = [(-1,0), (1,0), (0,1), (0,-1)];

                    let mut fails = init_fails;

                    for add in adds{
                        while fails > 0 {
                            p.y += add.0;
                            p.x += add.1;

                            let (ok, f) = fits_fail_limit(p, &field, fails);
                            //apparently you could just do let (ok, fails) = ... but that doesnt work here?
                            fails = f;

                            if !ok{
                                p.x = old_x;
                                p.y = old_y;
                                break;
                            }
                        }
                    }
                }

                if !is_piece_valid(p, &field){
                    p.matrix = old_matrix;
                }
            }

            if package.move_x != 0{
                p.x += package.move_x;
                if !is_piece_valid(p, &field){
                    p.x -= package.move_x;
                }
            }

            p.y += 1;


            let matrix_x = p.matrix.dim().1;
            let dropped = !is_piece_valid(p, &field);

            if dropped{
                //println!("dropped piece");
                p.y -= 1;
            }

            //put piece on the matrix
            for (i, b) in p.matrix.iter().enumerate().filter(|(_, b)| **b != Block::None){
                let x = i % matrix_x;
                let y = (i - x)/matrix_x;

                field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]] = *b;
            }

            if dropped{
                piece = None;
            }
        }

        //print screen
        //i should probably manually create the color prefix. having it on every block introduces a lot of overhead, esp on empty lines.
        //problem would then again be allocations so i'd have to go through the playing field to determine the amount of color changes.
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

                        assert_ne!(field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]], Block::None);
                        field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]] = Block::None;
                    } 
                }
            }
        }
        else {
            piece = Some(Piece::new(Block::VALUES[rand::random_range(0..(Block::VALUES.len()-1))], spawn_x, spawn_y));
        }


        
        let time = Instant::now();
        //debug_assert!(Duration::from_millis(2) > time.duration_since(last_frame), "{}", time.duration_since(last_frame).as_millis());
        if let Some(sleep_for) = frame_time.checked_sub(time.duration_since(last_frame)){

            if !sleep_for.is_zero() {
                sleep(sleep_for);
            }
        }

        last_frame = Instant::now();
    }
}

//fn is_block_valid(i: usize, b: &Block, matrix_x: usize, )

//this should bake the arguments into the closure as to avoid passing them and recalculating all the values for every single block, which would also involve
//accessing stuff from the shape of the matrix and thats slow
//i dont know what im doing, i just followed the compiler when it said to add lifetimes and a move (does make sense though)
fn get_block_validation_closure<'a>(piece: &'a Piece, field: &'a ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> Box<dyn Fn((usize, &Block)) -> bool + 'a>{
    let (_, matrix_x) = piece.matrix.dim();


    let(my, mx) = field.dim();
    let max_x = mx-1;
    let max_y = my-1;

    Box::new(
    move |(i, b)| *b == Block::None || {

    let tmp = i % matrix_x;
    let y = piece.y + ((i - tmp)/matrix_x) as i16;
    let x = tmp as i16 + piece.x;

    //yes, the y >= 0 check is necessary. i do not know why. most confusing thing is that this is only relevant
    //when the block reaches the bottom of the sreen so y should be like 39. dont care enough to think about it cause its working now.
    y <= max_y as i16 && x <= max_x as i16 && x >= 0 && y >= 0 && field[[y as usize, x as usize]] == Block::None
    })
}

use ndarray::OwnedRepr;

fn is_piece_valid(piece: &Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> bool{
    piece.matrix.iter().enumerate().all(get_block_validation_closure(piece, field))
}

fn fail_count(piece: &Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> usize{
    //mediator closure to call the validator because filter requires the arguments to be a reference
    //didnt come up with this myself, rust people on discord are pretty cool
    let validation_closure = get_block_validation_closure(piece, field);
    piece.matrix.iter().enumerate().filter(|(i,b)| {!validation_closure((*i,b))}).count()

    /*let mut fails = 0;
    let validation_closure = get_block_validation_closure(piece, field);
    for (i, b) in piece.matrix.iter().enumerate(){
        if validation_closure((i, b)){
            fails += 1;
        }
    }

    fails*/
}

//this short curcuits as to not view the entire matrix every time
fn fits_fail_limit(piece: &Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>, limit: usize) -> (bool, usize){
    
    let mut fails = 0;
    let validation_closure = get_block_validation_closure(piece, field);
    for (i, b) in piece.matrix.iter().enumerate(){
        if !validation_closure((i, b)){
            fails += 1;
            if fails >= limit{
                return (false, fails);
            }
        }
    }

    (true, fails)
}

/*
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
*/
