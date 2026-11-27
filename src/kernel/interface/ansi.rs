use super::{Cell, Color, MAX_COLUMNS, MAX_ROWS, Surface};
use core::fmt::Write;

pub struct AnsiSurface<W> {
    output: W,
    next: Option<(usize, usize)>,
    foreground: Option<Color>,
    cursor_hidden: bool,
}

impl<W: Write> AnsiSurface<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            next: None,
            foreground: None,
            cursor_hidden: false,
        }
    }

    pub fn finish(&mut self, x: usize, y: usize) {
        let _ = write!(self.output, "\x1b[0m\x1b[{};{}H", y + 1, x + 1);
        if self.cursor_hidden {
            let _ = self.output.write_str("\x1b[?25h");
            self.cursor_hidden = false;
        }
        self.foreground = None;
        self.next = Some((x, y));
    }
}

impl<W: Write> Surface for AnsiSurface<W> {
    fn draw_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x >= MAX_COLUMNS || y >= MAX_ROWS {
            return;
        }
        // Painting moves the host cursor through borders and content. Reveal it
        // only after finish() has returned it to the terminal's input position.
        if !self.cursor_hidden {
            let _ = self.output.write_str("\x1b[?25l");
            self.cursor_hidden = true;
        }
        if self.next != Some((x, y)) {
            let _ = write!(self.output, "\x1b[{};{}H", y + 1, x + 1);
        }
        if self.foreground != Some(cell.foreground) {
            // SGR 49 uses the terminal's actual default background. Palette
            // black can be a different shade and leave visible cell rectangles.
            let style = match cell.foreground {
                Color::Default => "\x1b[39;49m",
                Color::Blue => "\x1b[38;5;33;49m",
                Color::Green => "\x1b[38;5;40;49m",
                Color::Yellow => "\x1b[38;5;226;49m",
            };
            let _ = self.output.write_str(style);
            self.foreground = Some(cell.foreground);
        }
        if x == MAX_COLUMNS - 1 && y == MAX_ROWS - 1 {
            // Draw the desktop's final corner without scrolling the whole UI.
            // Restore wrapping immediately; no changed mode survives the frame.
            let _ = self.output.write_str("\x1b[?7l");
            let _ = self.output.write_char(cell.glyph);
            let _ = self.output.write_str("\x1b[?7h");
            self.next = None;
        } else {
            let _ = self.output.write_char(cell.glyph);
            self.next = Some((x + 1, y));
        }
    }
}
