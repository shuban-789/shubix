pub mod ansi;
pub mod input;
pub mod terminal;
pub mod window;

#[cfg(not(test))]
mod backend;
#[cfg(not(test))]
pub use backend::{putc, run_terminal, try_read_byte};

#[cfg(test)]
mod tests;

pub const MAX_COLUMNS: usize = 80;
pub const MAX_ROWS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Default,
    Blue,
    Green,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub glyph: char,
    pub foreground: Color,
}

impl Cell {
    pub const EMPTY: Self = Self::new(' ', Color::Default);

    pub const fn new(glyph: char, foreground: Color) -> Self {
        Self { glyph, foreground }
    }
}

pub trait Surface {
    fn draw_cell(&mut self, x: usize, y: usize, cell: Cell);
}
