//https://tetris.wiki/Tetris_Guideline
//https://tetris.fandom.com/wiki/SRS?file=SRS-pieces.png
//https://stackoverflow.com/questions/233850/tetris-piece-rotation-algorithm

mod block;
use block::Block;

mod piece;
use piece::Piece;

mod game;

mod input;
mod rendering;

use std::{
    io::{Write, stdout},
    thread,
};

use crossterm::{QueueableCommand, cursor, terminal};
use std::sync::{Arc, Mutex};

fn main() {
    let mut out = stdout();
    out.queue(cursor::MoveTo(0, 0))
        .expect("Should have been able to move cursor.")
        .queue(terminal::Clear(terminal::ClearType::FromCursorUp))
        .expect("Should have been able to clear.")
        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .expect("Should have been able to clear.")
        .flush()
        .expect("Should have been able to flush.");

    //the terminal has to be maximized because seemingly only text visible on screen is flushed
    println!("Maximize your terminal window. Then press enter.");

    let mut discard = "".to_string();
    std::io::stdin()
        .read_line(&mut discard)
        .expect("Failed to read line.");

    let package_access = Arc::new(Mutex::new(input::InputPackage::new()));
    let clone = package_access.clone();

    thread::spawn(move || input::activate(clone));

    loop {
        game::run(package_access.clone(), &mut out);

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
