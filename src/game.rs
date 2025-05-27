use crate::Block;
use crate::Piece;
use ndarray::prelude::*;
use ndarray::Array;
use std::time::{Duration, Instant};
use std::thread::sleep;

use crate::input::InputPackage;
use std::sync::{Arc, Mutex};



const FPS: u64 = 30;
const FRAME_TIME: Duration = Duration::from_millis(1000/FPS);

const RENDER_LINES: usize = 22;

pub fn run(package_access: Arc<Mutex<InputPackage>>){

    let spawn_x = 3;
    let spawn_y = 20;

    //option because there should be an update inbetween placing a piece and spawning the next one
    //because otherwise the player could maybe hard drop onto blocks that are being cleared that turn
    //also this lets me set it to none once dropped, making the rest of the loop a little simpler
    let mut piece: Option<Piece> = None;//Some(Piece::new(Block::Red, spawn_x, spawn_y));//None;

    let size_x = 10;
    let size_y = 40;
    let mut field = Array::<Block, _>::from_elem((size_y, size_x).f(), Block::None);

    let mut last_frame = Instant::now();

    let mut just_stored = false;
    let mut stored: Option<Array2<Block>> = None;

    let mut score: u128 = 0;

    let mut cleared_lines = 0;
    let mut lvl: u128 = 0;

    let mut ticks_per_grav_update: u16 = 10;
    let mut ticks_since_grav_update: u16 = 0;


    let mut bag: Vec<Block> = vec![];

    loop {

        let package = 
        {
            let mut mutex = package_access.lock().unwrap();
            let new_package = mutex.clone();
            *mutex = InputPackage::new();

            new_package
        };

        
        //2 ifs because thats how if let works ig
        if let Some(ref mut p) = piece {
            if package.store && !just_stored{

                if let Some(ref mut s) = stored{
                    std::mem::swap(s, &mut p.matrix);
                    p.x = spawn_x;
                    p.y = spawn_y;
                } else {
                    stored = Some(p.matrix.clone());
                    piece = None;
                }

                just_stored = true;
            }
        }


        //piece movement
        if let Some(ref mut p) = piece{

            //isolating this cause its a little longer
            perform_rotation(p, &field, package.rotate);

            //LR movement
            if package.move_x != 0{
                p.x += package.move_x;
                if !is_piece_valid(p, &field){
                    p.x -= package.move_x;
                }
            }


            let mut dropped = false;

            if package.hard_drop || ticks_since_grav_update >= ticks_per_grav_update{

                ticks_since_grav_update = ticks_since_grav_update.checked_sub(ticks_per_grav_update).or_else(|| Some(0)).unwrap();

                //falling
                let mut grav = 1;
                if package.hard_drop{
                    grav = size_y;
                }

                for _ in 0..grav {
                    p.y += 1;

                    dropped = !is_piece_valid(p, &field);

                    if dropped{
                        p.y -= 1;
                    }
                }
            }
            
            ticks_since_grav_update += if package.soft_drop {4} else {1};
            


            let matrix_x = p.matrix.dim().1;

            //put piece on the matrix
            for (i, b) in p.matrix.iter().enumerate().filter(|(_, b)| **b != Block::None){
                let x = i % matrix_x;
                let y = (i - x)/matrix_x;

                field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]] = *b;
            }

            if dropped{
                piece = None;
                just_stored = false;
                
                score += handle_line_clears(&mut field, &mut cleared_lines) as u128;
                if cleared_lines >= 10{
                    cleared_lines -= 10;
                    lvl += 1;

                    if lvl % 5 == 0{
                        ticks_per_grav_update = std::cmp::max(1, lvl as u16 -1 );
                    }
                }
            }
        }

        
        render(&field, score, lvl, &stored);

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

            if bag.is_empty(){
                bag = Block::VALUES.to_vec();
            }

            //have to do this with an if cause 0..0 panics
            let random = if bag.len() > 1 {rand::random_range(0..bag.len()-1)} else {0};
            let p = Piece::new(bag.remove(random), spawn_x, spawn_y);

            if !is_piece_valid(&p, &field){
                print!("{}[2J", 27 as char);
                let s = format!("{}{}{}", " ".repeat(size_x/2), format!("YOU LOST. SCORE: {}", score), " ".repeat(size_x/2));
                println!("{}", colored::Colorize::red(s.as_str()));
                return;
                //std::process::exit(0);
            }
            piece = Some(p);
        }


        
        let time = Instant::now();
        //debug_assert!(Duration::from_millis(2) > time.duration_since(last_frame), "{}", time.duration_since(last_frame).as_millis());
        if let Some(sleep_for) = FRAME_TIME.checked_sub(time.duration_since(last_frame)){

            if !sleep_for.is_zero() {
                sleep(sleep_for);
            }
        }

        last_frame = Instant::now();
    }
}


//width of a line is 20 chars
const RENDER_SIZE: usize = 2568;
fn render(field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>, score: u128, lvl: u128, stored: &Option<Array2<Block>>){
        //print screen
        //i should probably manually create the color prefix. having it on every block introduces a lot of overhead, esp on empty lines.
        //problem would then again be allocations so i'd have to go through the playing field to determine the amount of color changes.
        let mut out = String::with_capacity(RENDER_SIZE);//i just checked the size once, should probably do this mathematically

        let (size_y, size_x) = field.dim();

        out += &format!("Score: {}", score);
        out += &" ".repeat(19 - out.len());
        out += "\n";


        out += &format!("Level: {}", lvl);
        out += &" ".repeat(39 - out.len());
        out += "\n";


        let line = &format!("+{}+", &"-".repeat(size_x*2));
        out += line;

        let storage_display: Vec<String> = 
        if let Some(held) = stored{

            let mut s: Vec<String> = vec![];
            for row in held.rows().into_iter(){

                let mut r = String::with_capacity(35);
                for b in row{
                    r += &b.get_string_rep();
                }
                
                s.push(r);
            }

            s
        }
        else{
            vec!["None", "", "", ""].iter().map(|s| String::from(*s)).collect()
        };

        for (i, b) in field.iter().skip((size_y-RENDER_LINES) * size_x).enumerate(){
            out += &b.get_string_rep();
            if (i+1) % 10 == 0{
                out += if i > 10 {"|\n|"} else {"\n|"};
            }
        }
        out.pop();

        out += line;


        //this seemingly just fills up the screen with invisible chars, which is good enough i guess but i dont like it
        print!("{}[2J", 27 as char);

        println!("{}", out);

        println!("\nStorage:");
        if stored.is_some(){
            for s in storage_display{
                println!("{}", s);
            }
        }
        else {
            println!("Empty")
        }
}

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

use crate::input::Rotation;
fn perform_rotation(piece: &mut Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>, rotate: Rotation){
    
    if rotate != Rotation::Not{
        let rotated = piece.matrix_rotated(rotate);
        //ideally i'd do this without clone, i think std::mem::swap/take could work
        let old_matrix = piece.matrix.clone();
        piece.matrix = rotated;

        let init_fails = fail_count(piece, &field);
        let old_x = piece.x;
        let old_y = piece.y;

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
                    piece.y += add.0;
                    piece.x += add.1;

                    let (ok, f) = fits_fail_limit(piece, &field, fails);
                    //apparently you could just do let (ok, fails) = ... but that doesnt work here? like it just shadows fails
                    fails = f;

                    if !ok{
                        piece.x = old_x;
                        piece.y = old_y;
                        break;
                    }
                }
            }
        }

        if !is_piece_valid(piece, &field){
            piece.matrix = old_matrix;
        }
    }
}

fn handle_line_clears(field: &mut ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>, cleared_lines: &mut i32) -> u32{

    //turns out just swapping 2 rows in an ndarray is actually pretty complicated.
    //i tried using mem::swap but that requires 2 &mut to the rows, which violates
    //borrowing rules. instead i just decided to copy the entire thing if needed.
    //rust is fast and its just enum values and not strings. also it rarely happens

    let cleared: Vec<usize> =   field.rows().into_iter().enumerate()
                                //filter out empty rows
                                .filter(|(_, r)| r.iter().
                                    all(|b| *b != Block::None))
                                //remember them by their index
                                .map(|(i, _)| i)
                                .collect();
    
    let score = match cleared.len() {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,

        _ => panic!("Cannot clear more than 4 rows in one turn")
    };

    //fill cleared lines with Block::None
    for i in &cleared{
        let mut row = field.row_mut(*i);
        for j in 0..row.len(){
            row[j] = Block::None;
        }
    }

    if cleared.last().is_some(){

        let mut new_field = Array::<Block, _>::from_elem(field.dim().f(), Block::None);


        let mut new_rows = new_field.rows_mut().into_iter().rev();
        let mut new_row = new_rows.next().unwrap();

        for row in field.rows().into_iter().rev(){
            
            let mut advance = false;
            
            for (i, b) in row.iter().enumerate(){
                if *b != Block::None{
                    advance = true;
                }

                new_row[i] = *b;
            }

            //if there are only 'None' blocks in the old row then the new row should not change -> blocks sink to the bottom
            if advance{
                new_row = new_rows.next().unwrap();
            }
        }

        *field = new_field;
    }

    *cleared_lines += cleared.len() as i32;
    score
}