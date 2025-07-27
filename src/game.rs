use crate::Block;
use crate::Piece;
use crate::rendering;
use ndarray::Array;
use ndarray::OwnedRepr;
use ndarray::prelude::*;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::input::InputPackage;
use std::sync::{Arc, Mutex};

use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::{Stdout, Write};

use terminal::ClearType;

const FPS: u64 = 30;
const FRAME_TIME: Duration = Duration::from_millis(1000 / FPS);

const FIELD_SIZE_X: u8 = 10;
const FIELD_SIZE_Y: u8 = 40;
const RENDER_LINES: u8 = 22;
const RENDER_OFFSET: u8 = (FIELD_SIZE_Y - RENDER_LINES) - HEADER_SIZE;
const HEADER_SIZE: u8 = 2;
const RENDER_END: u8 = RENDER_LINES + HEADER_SIZE;
const RENDER_SIZE_X: u8 = FIELD_SIZE_X * 2 + 2;
const GAP: u8 = RENDER_SIZE_X - (2 + 4 * 2) * 2;
const SPAWN_X: u8 = 3;
const SPAWN_Y: u8 = 20;

pub fn run(package_access: Arc<Mutex<InputPackage>>, out: &mut Stdout) {
    //option because there should be an update inbetween placing a piece and spawning the next one.
    //otherwise the player could maybe hard drop onto blocks that are being cleared that turn
    //also this lets me set it to none once dropped, making the rest of the loop a little simpler
    let mut bag: Vec<Block> = vec![];
    let mut next = pull_piece(&mut bag, SPAWN_X, SPAWN_Y);
    let mut piece: Option<Piece> = Some(pull_piece(&mut bag, SPAWN_X, SPAWN_Y));

    let mut field = Array::<Block, _>::from_elem(
        (FIELD_SIZE_Y as usize, FIELD_SIZE_X as usize).f(),
        Block::Void,
    );

    let mut last_frame = Instant::now();

    let mut just_stored = false;
    let mut stored: Option<Piece> = None;

    let mut score: u128 = 0;
    let mut cleared_lines: u32 = 0;
    let mut lvl: u128 = 0;

    let mut ticks_per_grav_update: u16 = 10;
    let mut ticks_since_grav_update: u16 = 0;

    out.execute(cursor::Hide)
        .unwrap()
        .queue(terminal::Clear(terminal::ClearType::FromCursorUp))
        .expect("Should have been able to clear.")
        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .expect("Should have been able to clear.");

    render_static_elements(out);
    render_stats(score, lvl, out);
    render_views(&next, &None, out);
    new_render(&field, out);

    let mut redraw = true;

    //the spans used to determined what lines to redraw
    let mut old_span;
    let mut new_span;
    loop {
        old_span = Some(piece.as_ref().unwrap().get_span());
        if piece.as_ref().unwrap().y == 39 {
            print!("Uh")
        }

        //get input data
        let package = {
            let mut mutex = package_access.lock().unwrap();
            let new_package = mutex.clone();
            *mutex = InputPackage::new();

            new_package
        };

        //handle storing pieces
        if package.store && !just_stored && piece.is_some() {
            std::mem::swap(&mut stored, &mut piece);
            if let Some(ref mut p) = piece {
                p.x = SPAWN_X.into();
                p.y = SPAWN_Y.into();
            }

            just_stored = true;
            render_views(&next, &stored, out);
            redraw = true;
        }

        //piece movement
        if let Some(ref mut p) = piece {
            redraw |= p.perform_rotation(&field, package.rotate);

            //LR movement
            if package.move_x != 0 {
                p.move_by(package.move_x, 0);
                //*p += (package.move_x, 0);
                if !p.is_valid(&field) {
                    p.move_by(-package.move_x, 0);
                    //*p -= (package.move_x, 0);
                } else {
                    redraw = true;
                }
            }

            let mut dropped = false;

            if package.hard_drop || ticks_since_grav_update >= ticks_per_grav_update {
                redraw = true;

                ticks_since_grav_update = ticks_since_grav_update
                    .checked_sub(ticks_per_grav_update)
                    .or_else(|| Some(0))
                    .unwrap();

                //falling
                let mut grav = 1;
                if package.hard_drop {
                    grav = FIELD_SIZE_Y;
                }

                for _ in 0..grav {
                    p.y += 1;

                    if !p.is_valid(&field) {
                        dropped = true;
                        p.y -= 1;
                        break;
                    }
                }
            }

            ticks_since_grav_update += if package.soft_drop { 4 } else { 1 };

            p.add_to_field(&mut field);
            new_span = Some(p.get_span());

            if dropped {
                piece = None;
                just_stored = false;

                if handle_line_clears(&mut field, &mut score, &mut cleared_lines) {
                    if cleared_lines >= 10 {
                        cleared_lines -= 10;
                        lvl += 1;

                        //these stats aren't at all accurate to actual tetris but it is scaling
                        if lvl % 2 == 0 {
                            ticks_per_grav_update = ticks_per_grav_update
                                .checked_sub(1)
                                .or_else(|| Some(0))
                                .unwrap();

                            //old version that doesn't do anything until lvl 5 and then kills you
                            //ticks_per_grav_update = std::cmp::max(1, lvl as u16 - 1);
                        }
                    }

                    render_stats(score, lvl, out);
                    new_render(&field, out);
                    redraw = false;
                }
            }
        } else {
            new_span = None;
        }

        if redraw {
            /*
                render all lines the current piece covers

                if there was a piece before this turn:
                    render all lines it covers not covered by the current piece
            */

            let old_span = old_span.unwrap();
            for y in old_span.0..=old_span.1 {
                buffer_row_render(y, &field, out);
            }

            if let Some(new_span) = new_span {
                for y in (new_span.0..=new_span.1).filter(|y| *y < old_span.0 || *y > old_span.1) {
                    buffer_row_render(y, &field, out);
                }
            }
            out.flush().expect("Should have been able to flush.");
            redraw = false;
        }

        //remove piece from matrix or create new piece
        if let Some(ref p) = piece {
            p.remove_from_field(&mut field);
        } else {
            let mut p = pull_piece(&mut bag, SPAWN_X, SPAWN_Y);
            std::mem::swap(&mut p, &mut next);

            if !p.is_valid(&field) {
                out.queue(cursor::Show)
                    .expect("Should have been able to show cursor.")
                    .queue(terminal::Clear(terminal::ClearType::FromCursorUp))
                    .expect("Should have been able to clear.")
                    .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                    .expect("Should have been able to clear.")
                    //arbitrary line
                    .queue(cursor::MoveTo(0, RENDER_END.into()))
                    .expect("Should have been able to move cursor.");

                use crossterm::style::Stylize;
                println!(
                    "{}",
                    format!("GAME OVER.\nSCORE: {}\nLEVEL: {}", score, lvl)
                        .as_str()
                        .red()
                );
                return;
            }

            piece = Some(p);
            render_views(&next, &stored, out);
        }

        let time = Instant::now();
        if let Some(sleep_for) = FRAME_TIME.checked_sub(time.duration_since(last_frame)) {
            if !sleep_for.is_zero() {
                sleep(sleep_for);
            }
        }

        last_frame = Instant::now();
    }
}

fn pull_piece(bag: &mut Vec<Block>, x: u8, y: u8) -> Piece {
    if bag.is_empty() {
        *bag = Block::VALUES.to_vec();
    }

    //have to do this with an if cause 0..0 panics
    let random = if bag.len() > 1 {
        rand::random_range(0..bag.len() - 1)
    } else {
        0
    };

    Piece::new(bag.remove(random), x.into(), y.into())
}

///Redraws the whole field and flushes the buffered data.
fn new_render(field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>, out: &mut Stdout) {
    for y in FIELD_SIZE_Y - RENDER_LINES..FIELD_SIZE_Y {
        buffer_row_render(y, field, out);
    }
    out.flush().expect("Should have been able to flush.");
}

///Buffers rendering a row of the playing field. Does nothing when y is outside the bounds of the playing field.
fn buffer_row_render(
    y: u8,
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    out: &mut Stdout,
) {
    if y >= FIELD_SIZE_Y {
        return;
    }

    let mut bytes = vec![];
    //is this how you do this?
    bytes.append(&mut "|".as_bytes().to_vec());

    for b in field.row(y as usize) {
        bytes.append(&mut b.get_string_rep_colored().as_bytes().to_vec());
    }

    bytes.append(&mut "|".as_bytes().to_vec());

    rendering::buffer_line_render(
        (y - RENDER_OFFSET) as u16,
        &bytes,
        terminal::ClearType::CurrentLine,
        out,
    );
}

fn render_static_elements(out: &mut Stdout) {
    //upper and lower borders of playing field
    let dash = format!("+{}+", "-".repeat((RENDER_SIZE_X - 2) as usize));
    let bytes = dash.as_bytes();
    rendering::buffer_line_render(1, bytes, terminal::ClearType::CurrentLine, out);
    rendering::buffer_line_render(
        RENDER_END as u16,
        bytes,
        terminal::ClearType::CurrentLine,
        out,
    );

    //upper and lower borders + text for the piece view
    let y = RENDER_END + 3;
    let dash = format!("+{}+", "-".repeat(4 * 2));

    let word1 = "Next";
    let word2 = "Stored";
    let text = format!(
        "{}{}{}",
        word1,
        " ".repeat(dash.len() - word1.len() + GAP as usize),
        word2
    );
    rendering::buffer_line_render(y as u16 - 1, text.as_bytes(), ClearType::CurrentLine, out);

    let dash = format!("{}{}{}", dash, " ".repeat(GAP as usize), dash);

    let bytes = dash.as_bytes();

    rendering::buffer_line_render(y as u16, bytes, ClearType::CurrentLine, out);
    rendering::buffer_line_render(y as u16 + 5, bytes, ClearType::CurrentLine, out);
}

fn render_views(next: &Piece, stored: &Option<Piece>, out: &mut Stdout) {
    let beginning = RENDER_END + 4;

    for y in 0..4 {
        //construct byte array for next piece's string rep on this line
        let next_string: String = next.get_row_string(y);

        let stored_string: String = if let Some(p) = stored {
            p.get_row_string(y)
        } else {
            Block::Void.get_string_rep_colored().repeat(4)
        };

        let text = format!("|{}|{}|{}|", next_string, "  ".repeat(1), stored_string);
        let bytes = text.as_bytes();

        rendering::buffer_line_render(
            (beginning + y as u8) as u16,
            bytes,
            ClearType::CurrentLine,
            out,
        );
    }
}

///Buffers the new state of the level and score UI but does not flush it.
fn render_stats(score: u128, lvl: u128, out: &mut Stdout) {
    let score = format!("Score: {}", score.to_string());
    let lvl = format!("Lvl: {}", lvl.to_string());
    let wish_width = (FIELD_SIZE_X + 1) * 2;

    let line = format!(
        "{}{}{}",
        score,
        " ".repeat(std::cmp::max(
            wish_width as usize - (score.len() + lvl.len()),
            1
        )),
        lvl
    );

    rendering::buffer_line_render(0, line.as_bytes(), terminal::ClearType::CurrentLine, out);
}

///Clears any full lines in field and updates score and cleared_lines accordingly. Returns true if any lines were cleared, false otherwise.
fn handle_line_clears(
    field: &mut ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    score: &mut u128,
    cleared_lines: &mut u32,
) -> bool {
    //turns out just swapping 2 rows in an ndarray is actually pretty complicated.
    //i tried using mem::swap but that requires 2 &mut to the rows, which violates
    //borrowing rules. instead i just decided to copy the entire thing if needed.
    //rust is fast and its just enum values and not strings. also it rarely happens

    let cleared: Vec<usize> = field
        .rows()
        .into_iter()
        .enumerate()
        //filter out empty rows
        .filter(|(_, r)| r.iter().all(|b| *b != Block::Void))
        //remember them by their index
        .map(|(i, _)| i)
        .collect();

    *score += match cleared.len() {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,

        _ => panic!("Cannot clear more than 4 rows in one turn"),
    };

    //fill cleared lines with Block::None
    for i in &cleared {
        let mut row = field.row_mut(*i);
        for j in 0..row.len() {
            row[j] = Block::Void;
        }
    }

    if !cleared.is_empty() {
        let mut new_field = Array::<Block, _>::from_elem(field.dim().f(), Block::Void);

        let mut new_rows = new_field.rows_mut().into_iter().rev();
        let mut new_row = new_rows.next().unwrap();

        for row in field.rows().into_iter().rev() {
            let mut advance = false;

            for (i, b) in row.iter().enumerate() {
                if *b != Block::Void {
                    advance = true;
                }

                new_row[i] = *b;
            }

            //if there are only 'None' blocks in the old row then the new row should not change -> blocks sink to the bottom
            if advance {
                new_row = new_rows.next().unwrap();
            }
        }

        *field = new_field;
    }

    *cleared_lines += cleared.len() as u32;
    !cleared.is_empty()
}
