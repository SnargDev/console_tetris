//https://tetris.wiki/Tetris_Guideline
//https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
//https://stackoverflow.com/questions/233850/tetris-piece-rotation-algorithm

mod block;
use block::Block;

mod piece;
use piece::Piece;

mod game;

fn main() {
    game::run();
    //https://github.com/rust-ndarray/ndarray/blob/master/README-quick-start.md
}



/*
1. Rendering
-2d matrix to store color
-iterate over color matrix and render
 */