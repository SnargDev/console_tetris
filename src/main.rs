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

    let package_access = Arc::new(Mutex::new(input::InputPackage::new()));
    let clone = package_access.clone();

    thread::spawn(move || {
        input::activate(clone)
    });

    loop{
        game::run(package_access.clone());

        println!("Play again? Y/N");
        loop {
            use std::io;
            let mut input = String::new();
            io::stdin().read_line(&mut input)
            .expect("Failed to read line.");

            match input.trim().to_ascii_uppercase().as_str(){
                "Y" => break,
                "N" => std::process::exit(0),
                _ => println!("Invalid input '{}', expected Y or N.", input)
            }
        }
    }
}