use crossterm::{QueueableCommand, cursor, terminal};
use std::io::{Stdout, Write};

pub fn buffer_line_render(
    row: u16,
    bytes: &[u8],
    clear_type: terminal::ClearType,
    out: &mut Stdout,
) {
    out.queue(cursor::MoveTo(0, row))
        .expect("Should have been able to move cursor.")
        .queue(terminal::Clear(clear_type))
        .expect("Should have been able to clear line,")
        .write(bytes)
        .expect("Should have been able to write bytes to buffer.");
}
