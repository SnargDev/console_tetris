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

    game::run(package_access);

    //to terminate the input gathering thread instead of keeping the program in limbo forever
    std::process::exit(0);
    //https://github.com/rust-ndarray/ndarray/blob/master/README-quick-start.md
}



/*
1. Rendering
-2d matrix to store color
-iterate over color matrix and render
 */