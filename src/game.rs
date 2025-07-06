use crate::Block;
use crate::Piece;
use ndarray::Array;
use ndarray::prelude::*;
use std::io::Stdout;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::input::InputPackage;
use std::sync::{Arc, Mutex};

use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::{Write, stdout};

use crossterm::style;
use style::Stylize;

const FPS: u64 = 30;
const FRAME_TIME: Duration = Duration::from_millis(1000 / FPS);

const RENDER_LINES: u8 = 22;

const FIELD_SIZE_X: u8 = 10;
const FIELD_SIZE_Y: u8 = 40;
//the index of the first row in field that is supposed to be rendered.
const RENDER_ORIGIN: u8 = (FIELD_SIZE_Y - RENDER_LINES) + 2;

/*
When does the screen have to be updated?

Game:
    piece moved left or right
    piece affected by gravity
        is soft drop held?
    piece hard dropped
    piece rotated
    lines cleared

UI:
    score changed
    level changed
    stored piece changed


===========================
How should lines be updated?

Option 1:
    everything flushed immediately
    may produce flickering idk

Option 2:
    array of strings
    if anything has changed, iterate over all non-empty lines and buffer their text

 */

pub fn run(package_access: Arc<Mutex<InputPackage>>, use_color: bool) {
    //initialize out, hide cursor, clear everything
    let mut out = stdout();
    out.execute(cursor::Hide).unwrap();
    out.queue(terminal::Clear(terminal::ClearType::FromCursorUp))
        .unwrap();
    out.queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .unwrap();

    print!(
        "{}{}",
        format!("|{}|\n", "[]".repeat(FIELD_SIZE_X.into())).repeat(RENDER_LINES.into()),
        format!("+{}+", "-".repeat(2 * FIELD_SIZE_X as usize))
    );

    let spawn_x = 3;
    let spawn_y = 20;

    //option because there should be an update inbetween placing a piece and spawning the next one.
    //otherwise the player could maybe hard drop onto blocks that are being cleared that turn
    //also this lets me set it to none once dropped, making the rest of the loop a little simpler
    let mut piece: Option<Piece> = None;

    let mut field = Array::<Block, _>::from_elem(
        (FIELD_SIZE_Y as usize, FIELD_SIZE_X as usize).f(),
        Block::None,
    );

    let mut last_frame = Instant::now();

    let mut just_stored = false;
    let mut stored: Option<Array2<Block>> = None;

    let mut score: u128 = 0;

    let mut clear_count = 0;
    let mut lvl: u128 = 0;

    let mut ticks_per_grav_update: u16 = 10;
    let mut ticks_since_grav_update: u16 = 0;

    let mut bag: Vec<Block> = vec![];

    loop {
        let package = {
            let mut mutex = package_access.lock().unwrap();
            let new_package = mutex.clone();
            *mutex = InputPackage::new();

            new_package
        };

        let mut redraw = false;

        //0 is the old y level
        //1 is the height of the old matrix
        let old_stats: Option<(u8, u8)> = match piece {
            Some(ref p) => Some((p.y as u8, p.matrix.dim().0 as u8)),
            None => None,
        };

        //2 ifs because thats how if let works ig
        if let Some(ref mut p) = piece {
            if package.store && !just_stored {
                if let Some(ref mut s) = stored {
                    std::mem::swap(s, &mut p.matrix);
                    p.x = spawn_x;
                    p.y = spawn_y;
                } else {
                    stored = Some(p.matrix.clone());
                    piece = None;
                }

                just_stored = true;
                redraw = true;
            }
        }

        //piece movement
        if let Some(ref mut p) = piece {
            redraw |= perform_rotation(p, &field, package.rotate);

            //LR movement
            if package.move_x != 0 {
                *p += (package.move_x, 0);
                if !is_piece_valid(p, &field) {
                    *p -= (package.move_x, 0);
                } else {
                    redraw = true;
                }
            }

            let mut dropped = false;

            if package.hard_drop || ticks_since_grav_update >= ticks_per_grav_update {
                ticks_since_grav_update = ticks_since_grav_update
                    .checked_sub(ticks_per_grav_update)
                    .or_else(|| Some(0))
                    .unwrap();

                //falling
                let grav = if package.hard_drop { FIELD_SIZE_Y } else { 1 };
                for _ in 0..grav {
                    p.y += 1;

                    dropped = !is_piece_valid(p, &field);

                    if dropped {
                        p.y -= 1;
                    }
                }

                redraw = true;
            }

            ticks_since_grav_update += if package.soft_drop { 4 } else { 1 };

            /*if *p != old {
                let matrix_x = p.matrix.dim().1;

                //remove old blocks from the screen
                for (i, _) in old
                    .matrix
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| **b == Block::None)
                {
                    let x = i % matrix_x;
                    let y = (i - x) / matrix_x;

                    /*
                    //x2 cause 1 block == [] and because there is a pipe symbol at the start of the line
                    let x = (p.x + x as i16) * 2 + 1;
                    let y = p.y + y as i16 + 2;

                    out.queue(cursor::MoveTo(x as u16, y as u16))
                        .expect("Should have been able to move the cursor.");
                    out.write("  ".as_bytes())
                        .expect("Should have been able to write to the buffer.");
                    */
                }

                //put new matrix on the screen
                for (i, b) in p
                    .matrix
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| **b != Block::None)
                {
                    let x = i % matrix_x;
                    let y = (i - x) / matrix_x;

                    //x2 cause 1 block == [] and because there is a pipe symbol at the start of the line
                    let x = (p.x + x as i16) * 2 + 1;
                    let y = p.y + y as i16 + 2;

                    out.queue(cursor::MoveTo(x as u16, y as u16))
                        .expect("Should have been able to move the cursor.")
                        .write(b.get_string_rep_colored().as_bytes())
                        //.queue(style::PrintStyledContent(b.as_styled_comment()))
                        .expect("Should have been able to write to the buffer.");
                }

                //REAL
                //render_lines(vec![(p.y - 1).try_into().unwrap()], &field, &mut out);

                out.flush().expect("Should have been able to flush.");
            } else {
                println!("Nothing changed, this is supposed to happen sometimes.");
                return;
            }*/

            let matrix_x = p.matrix.dim().1;

            //put piece on the matrix
            //then check for line clears
            //then redraw the screen if redraw is true
            //then removed the piece again if it wasn't dropped
            //the game should ideally only check do after a gravity update
            for (i, b) in p
                .matrix
                .iter()
                .enumerate()
                .filter(|(_, b)| **b != Block::None)
            {
                let x = i % matrix_x;
                let y = (i - x) / matrix_x;

                field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]] = *b;
            }

            if dropped {
                piece = None;
                just_stored = false;

                let cleared = handle_line_clears(&mut field);
                clear_count += cleared.len() as i32;
                score += get_score_for_lines(cleared.len() as u8);
                //redraw |= !cleared.is_empty();

                //update level
                if clear_count >= 10 as i32 {
                    clear_count -= 10 as i32;
                    lvl += 1;

                    if lvl % 5 == 0 {
                        ticks_per_grav_update = std::cmp::max(1, lvl as u16 - 1);
                    }
                }

                if !cleared.is_empty() {
                    let lines: Vec<u8> = (0..FIELD_SIZE_Y).collect();
                    render_lines(&lines, &field, &mut out);
                    //this is temporary. since we redraw the whole field after lines are cleared, we dont need to redraw anything else later on.
                    redraw = false;
                }

                //nah
                if redraw {
                    /*
                       for all lines the old piece covered and that are not covered by the current piece
                           redraw

                       for all lines that were cleared and that are not covered by the current piece
                           redraw

                       for all lines covered by the current piece
                           redraw
                    */

                    /*for (y in old_y..old_y + old_my).filter(y < p.y || y > p.y + my) //all lines not inside
                        redraw line
                    */
                }
            }
        }

        //remove piece from the matrix
        if let Some(ref p) = piece {
            let (sy, sx) = p.matrix.dim();

            for y in 0..sy {
                for x in 0..sx {
                    let b = p.matrix[[y, x]];
                    if b != Block::None {
                        assert_ne!(
                            field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]],
                            Block::None
                        );

                        field[[(y as i16 + p.y) as usize, (x as i16 + p.x) as usize]] = Block::None;
                    }
                }
            }
        } else {
            if bag.is_empty() {
                bag = Block::VALUES.to_vec();
            }

            //have to do this with an if cause 0..0 panics
            let random = if bag.len() > 1 {
                rand::random_range(0..bag.len() - 1)
            } else {
                0
            };
            let p = Piece::new(bag.remove(random), spawn_x, spawn_y);

            if !is_piece_valid(&p, &field) {
                //fixme: ununwrap
                out.queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                    .unwrap();
                out.queue(terminal::Clear(terminal::ClearType::FromCursorUp))
                    .unwrap();
                out.flush().unwrap();

                let s = format!(
                    "{}{}{}",
                    " ".repeat(FIELD_SIZE_X as usize / 2),
                    format!("Game over. Score: {}", score),
                    " ".repeat(FIELD_SIZE_X as usize / 2)
                );

                println!("{}", s.red());
                return;
            }
            piece = Some(p);
        }

        if redraw {
            /*
            1. if there WAS a piece:
                draw all lines of the matrix for the old position until you reach the position of the current piece, if it exists

            2. if there IS a piece:
                draw all lines of the matrix of the current piece

            3. else (for 2) for all lines that were cleared and do not overlap with the (old?) matrix of the piece, redraw them
                FOR NOW: if any lines have been cleared, redraw everything.
             */

            //1
            if let Some((old_y, old_height)) = old_stats {
                let lines: Vec<u8> = if let Some(ref p) = piece {
                    let height = p.matrix.dim().0;
                    (old_y..old_y + old_height)
                        .filter(|y| *y < p.y as u8 || *y > p.y as u8 + height as u8)
                        .collect()
                } else {
                    (old_y..old_y + old_height).collect()
                };

                buffer_multi_line_render(&lines, &field, &mut out);
            }

            //2
            if let Some(ref p) = piece {
                let lines: Vec<u8> = (p.y as u8..p.y as u8 + p.matrix.dim().0 as u8).collect();
                buffer_multi_line_render(&lines, &field, &mut out);
            }

            //todo!("Implement redrawing");
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

//this should bake the arguments into the closure as to avoid passing them and recalculating all the values for every single block, which would also involve
//accessing stuff from the shape of the matrix and thats slow
fn get_block_validation_closure<'a>(
    piece: &'a Piece,
    field: &'a ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
) -> Box<dyn Fn((usize, &Block)) -> bool + 'a> {
    let (_, matrix_x) = piece.matrix.dim();

    let (my, mx) = field.dim();
    let max_x = mx - 1;
    let max_y = my - 1;

    Box::new(move |(i, b)| {
        *b == Block::None || {
            let tmp = i % matrix_x;
            let y = piece.y + ((i - tmp) / matrix_x) as i16;
            let x = tmp as i16 + piece.x;

            //yes, the y >= 0 check is necessary. i do not know why. most confusing thing is that this is only relevant
            //when the block reaches the bottom of the sreen so y should be like 39. dont care enough to think about it cause its working now.
            y <= max_y as i16
                && x <= max_x as i16
                && x >= 0
                && y >= 0
                && field[[y as usize, x as usize]] == Block::None
        }
    })
}

use ndarray::OwnedRepr;

fn is_piece_valid(piece: &Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> bool {
    piece
        .matrix
        .iter()
        .enumerate()
        .all(get_block_validation_closure(piece, field))
}

fn fail_count(piece: &Piece, field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> usize {
    //mediator closure to call the validator because filter requires the arguments to be a reference
    //didnt come up with this myself, rust people on discord are pretty cool
    let validation_closure = get_block_validation_closure(piece, field);
    piece
        .matrix
        .iter()
        .enumerate()
        .filter(|(i, b)| !validation_closure((*i, b)))
        .count()
}

//this short curcuits as to not view the entire matrix every time
fn fits_fail_limit(
    piece: &Piece,
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    limit: usize,
) -> (bool, usize) {
    let mut fails = 0;
    let validation_closure = get_block_validation_closure(piece, field);
    for (i, b) in piece.matrix.iter().enumerate() {
        if !validation_closure((i, b)) {
            fails += 1;
            if fails >= limit {
                return (false, fails);
            }
        }
    }

    (true, fails)
}

use crate::input::Rotation;
///rotates the matrix of the piece and returns whether a rotation occured or not
fn perform_rotation(
    piece: &mut Piece,
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    rotate: Rotation,
) -> bool {
    if rotate == Rotation::Not {
        return false;
    }

    let rotated = piece.matrix_rotated(rotate);
    //ideally i'd do this without clone, i think std::mem::swap/take could work
    let old_matrix = piece.matrix.clone();
    piece.matrix = rotated;

    let init_fails = fail_count(piece, &field);
    let old_x = piece.x;
    let old_y = piece.y;

    //idk if these wall kicks are 100% accurate but im not going to read through half the tetris wiki and hardcode all the values for every piece, i like this
    if init_fails > 0 {
        //while fail count is decreasing
        //  move block
        // if fail count == 0 => done else back to initial position
        //if the only condition is for the fail count to <= prev then a piece could force itself through other blocks

        //up, down, right, left
        let adds = [(0, -1), (0, 1), (1, 0), (-1, 0)];

        let mut fails = init_fails;

        for add in adds {
            while fails > 0 {
                *piece += add;

                let (ok, f) = fits_fail_limit(piece, &field, fails);
                //apparently you could just do let (ok, fails) = ... but that doesnt work here? like it just shadows fails
                fails = f;

                if !ok {
                    piece.x = old_x;
                    piece.y = old_y;
                    break;
                }
            }
        }
    }

    if !is_piece_valid(piece, &field) {
        piece.matrix = old_matrix;
        return false;
    }

    return true;
}

//returns a vec instead of &[u8] because that demands lifetime annotations even though nothing is directly taken from the parameters and still doesnt work
fn handle_line_clears(field: &mut ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>) -> Vec<usize> {
    //turns out just swapping 2 rows in an ndarray is actually pretty complicated.
    //i tried using mem::swap but that requires 2 &mut to the rows, which violates
    //borrowing rules. instead i just decided to copy the entire thing if needed.
    //rust is fast and its just enum values and not strings. also it rarely happens

    let mut changed: Vec<usize> = vec![];

    let cleared: Vec<usize> = field
        .rows()
        .into_iter()
        .enumerate()
        //filter out empty rows
        .filter(|(_, r)| r.iter().all(|b| *b != Block::None))
        //remember them by their index
        .map(|(i, _)| i)
        .collect();

    //fill cleared lines with Block::None
    for i in &cleared {
        let mut row = field.row_mut(*i);
        for j in 0..row.len() {
            row[j] = Block::None;
        }
    }

    if !cleared.is_empty() {
        let mut new_field = Array::<Block, _>::from_elem(field.dim().f(), Block::None);

        let mut new_rows = new_field.rows_mut().into_iter().rev();
        let mut new_row = new_rows.next().unwrap();

        //let mut first = -1;
        //let mut last = -1;

        for (y, row) in field.rows().into_iter().enumerate().rev() {
            let mut advance = false;

            for (i, b) in row.iter().enumerate() {
                if *b != Block::None {
                    advance = true;
                }

                new_row[i] = *b;
            }

            //if there are only 'None' blocks in the old row then the new row should not change -> blocks sink to the bottom
            if advance {
                new_row = new_rows.next().unwrap();

                //FIXME
                //maybe this is correct? idk
                changed.push(y);
            }
        }

        *field = new_field;
    }

    return changed;
}

fn get_score_for_lines(lines: u8) -> u128 {
    //this does not account for that minor point bonus you get depending on what height you dropped a piece on but it really doesnt matter because of how small that bonus is
    match lines {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,

        _ => panic!("Cannot clear more than 4 rows in one turn"),
    }
}

/*fn render_lines(
    lines: Vec<u8>,
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    out: &mut Stdout,
) {
    for l in lines {
        let y = (l - RENDER_ORIGIN) + 2;

        out.queue(cursor::MoveTo(0, y.into()))
            .expect("Should have been able to move cursor.");
        out.queue(terminal::Clear(terminal::ClearType::CurrentLine))
            .expect("Should have been able to clear terminal.");

        for x in 0..FIELD_SIZE_X {
            out.write(
                if field[[y as usize, x as usize]] == Block::None {
                    "  "
                } else {
                    "[]"
                }
                .as_bytes(),
            )
            .expect("Should have been able to write to buffer.");
        }

        out.flush().expect("Should have been able to flush.");
    }
}*/

fn buffer_line_render(
    y: u8,
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    out: &mut Stdout,
) {
    println!("Line render!!!!!");
    println!("y: {}", y);

    out.queue(cursor::MoveTo(0, (y + RENDER_ORIGIN) as u16))
        .expect("Should have been able to move cursor.")
        .queue(terminal::Clear(terminal::ClearType::CurrentLine))
        .expect("Should have been able to clear line,");

    let mut bytes = vec![];
    //is this how you do this?
    bytes.append(&mut "|".as_bytes().to_vec());

    for b in field.row(y as usize) {
        bytes.append(&mut b.get_string_rep_colored().as_bytes().to_vec());
    }

    bytes.append(&mut "|".as_bytes().to_vec());

    out.write(&bytes)
        .expect("Should have been able to write bytes to buffer.");
}

fn buffer_multi_line_render(
    lines: &[u8],
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    out: &mut Stdout,
) {
    for y in lines {
        buffer_line_render(*y, field, out);
    }
}

fn render_lines(
    lines: &[u8],
    field: &ArrayBase<OwnedRepr<Block>, Dim<[usize; 2]>>,
    out: &mut Stdout,
) {
    buffer_multi_line_render(lines, field, out);
    out.flush().expect("Should have been able to flush buffer.");
}

fn testing_bs(mut out: Stdout) {
    //testing code for colored printing
    sleep(Duration::from_secs(2));
    out.queue(cursor::MoveTo(0, 15)).unwrap();
    out.queue(terminal::Clear(terminal::ClearType::CurrentLine))
        .unwrap();

    let line = format!(
        "|{}|",
        Block::DarkBlue
            .get_string_rep_colored()
            .repeat(FIELD_SIZE_X.into())
    );

    out.write(line.as_bytes()).unwrap();
    out.flush().unwrap();

    //

    out.queue(cursor::MoveTo(0, 15)).unwrap();
    out.queue(terminal::Clear(terminal::ClearType::CurrentLine))
        .unwrap();

    let mut line = format!(
        "|{}|",
        Block::Red
            .get_string_rep_colored()
            .repeat(FIELD_SIZE_X.into())
    );

    out.write(line.as_bytes()).unwrap();
    out.flush().unwrap();

    out.queue(terminal::Clear(terminal::ClearType::CurrentLine))
        .unwrap();

    out.queue(cursor::MoveTo(0, 15)).unwrap();

    let lower = 4;
    let upper = 5;
    //let mut vec: Vec<char> = Vec::with_capacity(FIELD_SIZE_X as usize * 2 + 2);

    /*let new_line: String = line
    .chars()
    .enumerate()
    .map(|(i, c)| {
        if i >= lower && i <= upper {
            if i % 2 == 0 { '[' } else { ']' }
        } else {
            c
        }
    })
    .collect();*/

    //THIS SEEMS TO BE CORRECT
    //just replace the bounds check with 'if x is a valid position in p's matrix AND the block at that position is not none'
    let mut vec: Vec<u8> = vec![b'|'];

    for x in 0..FIELD_SIZE_X {
        if x >= lower && x <= upper {
            vec.extend_from_slice(Block::Yellow.get_string_rep_colored().as_bytes());
        } else {
            vec.push(b'[');
            vec.push(b']');
        }
    }
    vec.push(b'|');

    out.write(&vec).unwrap();

    //let new_line: String = vec.iter().collect();
    //out.write(new_line.as_bytes()).unwrap();

    /*for (i, char) in line.chars().enumerate() {
        if (lower..upper).contains(&(i as u8)) {
            let char: char = if i % 2 == 0 { '[' } else { ']' };
            vec.push(char);
            //out.write(char.to_string().as_bytes()).unwrap();
        } else {
            vec.push(char);
            //out.write(char.to_string().as_bytes()).unwrap();
        }
    }*/

    //let s: String = vec.iter().collect();
    //out.write(s.as_bytes()).unwrap();

    //print!("{}", Block::DarkBlue.as_styled_comment().to_string());
    return;
}
