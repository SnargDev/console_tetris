//https://tetris.wiki/Tetris_Guideline
//https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
//https://stackoverflow.com/questions/233850/tetris-piece-rotation-algorithm

mod block;
use block::Block;

mod piece;
use piece::Piece;

mod game;

mod input;

use std::thread;

use std::sync::{Arc, Mutex};

fn main() {
    //TODO: remove this, use TermFeatures::has_color instead
    //println!("Use color?");
    //let use_color = get_yn_inp();

    let package_access = Arc::new(Mutex::new(input::InputPackage::new()));
    let clone = package_access.clone();

    thread::spawn(move || input::activate(clone));

    loop {
        //fix up all this use color stuff
        game::run(package_access.clone(), true); //use_color);

        println!("Play again? Y/N");
        if !get_yn_inp() {
            std::process::exit(0);
        }
    }
}

fn get_yn_inp() -> bool {
    loop {
        use std::io;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");
        input = input.trim().to_ascii_uppercase();

        match input.as_str() {
            "Y" => return true,
            "N" => return false,
            _ => println!("Invalid input '{}', expected Y or N.", input),
        }
    }
}
