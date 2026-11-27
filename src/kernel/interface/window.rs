use super::{Cell, Color, MAX_COLUMNS, MAX_ROWS, Surface, input::Key, terminal::Terminal};

const MIN_WIDTH: usize = 12;
const MIN_HEIGHT: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Shell,
    Resize,
    Move,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct Desktop {
    pub terminal: Terminal,
    bounds: Rect,
    columns: usize,
    rows: usize,
    mode: Mode,
    previous: [Cell; MAX_COLUMNS * MAX_ROWS],
    painted: bool,
}

impl Desktop {
    pub const fn new() -> Self {
        Self {
            terminal: Terminal::new(),
            bounds: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            columns: 0,
            rows: 0,
            mode: Mode::Shell,
            previous: [Cell::EMPTY; MAX_COLUMNS * MAX_ROWS],
            painted: false,
        }
    }

    pub fn init(&mut self, columns: usize, rows: usize) -> bool {
        if columns < MIN_WIDTH + 2 || rows < MIN_HEIGHT + 2 {
            return false;
        }
        self.columns = columns.min(MAX_COLUMNS);
        self.rows = rows.min(MAX_ROWS);
        self.bounds = Rect {
            x: 1,
            y: 1,
            width: self.columns - 2,
            height: self.rows - 2,
        };
        self.mode = Mode::Shell;
        self.painted = false;
        self.terminal
            .resize(self.bounds.width - 2, self.bounds.height - 2);
        true
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }
    pub fn resizing(&self) -> bool {
        self.mode == Mode::Resize
    }
    pub fn moving(&self) -> bool {
        self.mode == Mode::Move
    }

    /// Return only input intended for the shell. Window commands never reach it.
    pub fn key(&mut self, key: Key) -> Option<u8> {
        let requested_mode = match key {
            Key::Resize => Some(Mode::Resize),
            Key::Move => Some(Mode::Move),
            _ => None,
        };
        if let Some(mode) = requested_mode {
            self.mode = if self.mode == mode { Mode::Shell } else { mode };
            return None;
        }
        if self.mode == Mode::Shell {
            return match key {
                Key::Byte(byte) => Some(byte),
                _ => None,
            };
        }
        let old = self.bounds;
        if matches!(key, Key::Byte(b'\r' | b'\n')) {
            self.mode = Mode::Shell;
        } else if self.mode == Mode::Move {
            match key {
                Key::Left => self.bounds.x = self.bounds.x.saturating_sub(1).max(1),
                Key::Right => {
                    self.bounds.x = (self.bounds.x + 1).min(self.columns - self.bounds.width - 1)
                }
                Key::Up => self.bounds.y = self.bounds.y.saturating_sub(1).max(1),
                Key::Down => {
                    self.bounds.y = (self.bounds.y + 1).min(self.rows - self.bounds.height - 1)
                }
                _ => {}
            }
        } else {
            match key {
                Key::Left => self.bounds.width = self.bounds.width.saturating_sub(1).max(MIN_WIDTH),
                Key::Right => {
                    self.bounds.width =
                        (self.bounds.width + 1).min(self.columns - self.bounds.x - 1)
                }
                Key::Up => {
                    self.bounds.height = self.bounds.height.saturating_sub(1).max(MIN_HEIGHT)
                }
                Key::Down => {
                    self.bounds.height = (self.bounds.height + 1).min(self.rows - self.bounds.y - 1)
                }
                _ => {}
            }
        }
        if old.width != self.bounds.width || old.height != self.bounds.height {
            self.terminal
                .resize(self.bounds.width - 2, self.bounds.height - 2);
        }
        None
    }

    /// Recompute the composition, drawing only changed cells. This also erases
    /// old borders after shrinking; neither shell output nor scroll can touch them.
    pub fn render(&mut self, surface: &mut impl Surface) {
        for y in 0..self.rows {
            for x in 0..self.columns {
                let cell = self.compose(x, y);
                let index = y * MAX_COLUMNS + x;
                if !self.painted || self.previous[index] != cell {
                    surface.draw_cell(x, y, cell);
                    self.previous[index] = cell;
                }
            }
        }
        self.painted = true;
    }

    fn compose(&self, x: usize, y: usize) -> Cell {
        // Desktop decoration is outside every application's bounds. It never
        // takes focus or changes color when the terminal window is resized.
        if x == 0 || y == 0 || x == self.columns - 1 || y == self.rows - 1 {
            let glyph = match (x, y) {
                (0, 0) => '┏',
                (x, 0) if x == self.columns - 1 => '┓',
                (0, y) if y == self.rows - 1 => '┗',
                (x, y) if x == self.columns - 1 && y == self.rows - 1 => '┛',
                (0, _) => '┃',
                (x, _) if x == self.columns - 1 => '┃',
                _ => '━',
            };
            return Cell::new(glyph, Color::Blue);
        }
        let r = self.bounds;
        if x < r.x || x >= r.x + r.width || y < r.y || y >= r.y + r.height {
            return Cell::EMPTY;
        }
        let (cx, cy) = (x - r.x, y - r.y);
        let glyph = match (cx, cy) {
            (0, 0) => '┏',
            (x, 0) if x == r.width - 1 => '┓',
            (0, y) if y == r.height - 1 => '┗',
            (x, y) if x == r.width - 1 && y == r.height - 1 => '┛',
            (0, _) => '┃',
            (x, _) if x == r.width - 1 => '┃',
            (_, 0) => '━',
            (_, y) if y == r.height - 1 => '━',
            _ => {
                let (tx, ty) = (cx - 1, cy - 1);
                return Cell::new(self.terminal.cell(tx, ty) as char, Color::Default);
            }
        };
        let fg = match self.mode {
            Mode::Shell => Color::Blue,
            Mode::Resize => Color::Green,
            Mode::Move => Color::Yellow,
        };
        let title = b" Terminal ";
        if cy == 0 && cx >= 1 && cx <= title.len() && cx < r.width - 1 {
            return Cell::new(title[cx - 1] as char, fg);
        }
        Cell::new(glyph, fg)
    }
}
