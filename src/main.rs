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
use std::sync::mpsc;
fn main() {

    let (inp_t, inp_r) = mpsc::channel();

    thread::spawn(move || {
        input::activate(inp_t)
    });

    game::run(inp_r);

    //to terminate the input gathering thread instead of keeping the program in limbo forever
    std::process::exit(0);
    //https://github.com/rust-ndarray/ndarray/blob/master/README-quick-start.md
}



/*
1. Rendering
-2d matrix to store color
-iterate over color matrix and render
 */