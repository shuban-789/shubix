use super::{MAX_COLUMNS, MAX_ROWS};

const HISTORY: usize = 4096;

#[derive(Clone, Copy)]
enum Escape {
    Ground,
    Start,
    Csi {
        params: [usize; 2],
        index: usize,
        invalid: bool,
    },
}

pub struct Terminal {
    cells: [u8; MAX_COLUMNS * MAX_ROWS],
    columns: usize,
    rows: usize,
    x: usize,
    y: usize,
    history: [u8; HISTORY],
    start: usize,
    length: usize,
    escape: Escape,
}

impl Terminal {
    pub const fn new() -> Self {
        Self {
            cells: [b' '; MAX_COLUMNS * MAX_ROWS],
            columns: 1,
            rows: 1,
            x: 0,
            y: 0,
            history: [0; HISTORY],
            start: 0,
            length: 0,
            escape: Escape::Ground,
        }
    }

    pub fn resize(&mut self, columns: usize, rows: usize) {
        self.columns = columns.clamp(1, MAX_COLUMNS);
        self.rows = rows.clamp(1, MAX_ROWS);
        self.cells.fill(b' ');
        self.x = 0;
        self.y = 0;
        self.escape = Escape::Ground;
        for i in 0..self.length {
            self.apply(self.history[(self.start + i) % HISTORY]);
        }
    }

    pub fn putc(&mut self, byte: u8) {
        if self.length == HISTORY {
            self.start = (self.start + 1) % HISTORY;
            self.length -= 1;
        }
        self.history[(self.start + self.length) % HISTORY] = byte;
        self.length += 1;
        self.apply(byte);
    }

    pub fn cell(&self, x: usize, y: usize) -> u8 {
        if x < self.columns && y < self.rows {
            self.cells[y * MAX_COLUMNS + x]
        } else {
            b' '
        }
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.x.min(self.columns - 1), self.y)
    }

    fn newline(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y == self.rows {
            self.cells
                .copy_within(MAX_COLUMNS..self.rows * MAX_COLUMNS, 0);
            self.cells[(self.rows - 1) * MAX_COLUMNS..self.rows * MAX_COLUMNS].fill(b' ');
            self.y -= 1;
        }
    }

    fn apply(&mut self, byte: u8) {
        if byte == 0x1b {
            self.escape = Escape::Start;
            return;
        }
        match self.escape {
            Escape::Start => {
                self.escape = if byte == b'[' {
                    Escape::Csi {
                        params: [0; 2],
                        index: 0,
                        invalid: false,
                    }
                } else {
                    Escape::Ground
                };
                return;
            }
            Escape::Csi {
                mut params,
                mut index,
                mut invalid,
            } => {
                match byte {
                    b'0'..=b'9' => {
                        params[index] = params[index]
                            .saturating_mul(10)
                            .saturating_add((byte - b'0') as usize);
                    }
                    b';' if index == 0 => index = 1,
                    0x40..=0x7e => {
                        self.escape = Escape::Ground;
                        if !invalid {
                            self.csi(byte, params, index);
                        }
                        return;
                    }
                    _ => invalid = true,
                }
                self.escape = Escape::Csi {
                    params,
                    index,
                    invalid,
                };
                return;
            }
            Escape::Ground => {}
        }
        match byte {
            b'\r' => self.x = 0,
            b'\n' => self.newline(),
            8 | 127 => {
                if self.x > 0 {
                    self.x -= 1;
                } else if self.y > 0 {
                    self.y -= 1;
                    self.x = self.columns - 1;
                } else {
                    return;
                }
                self.cells[self.y * MAX_COLUMNS + self.x] = b' ';
            }
            b'\t' => {
                for _ in 0..4 - self.x % 4 {
                    self.apply(b' ');
                }
            }
            0..=31 => {}
            _ => {
                if self.x == self.columns {
                    self.newline();
                }
                self.cells[self.y * MAX_COLUMNS + self.x] = if byte < 127 { byte } else { b'?' };
                self.x += 1;
            }
        }
    }

    fn csi(&mut self, command: u8, params: [usize; 2], index: usize) {
        match command {
            b'H' | b'f' => {
                self.y = params[0].saturating_sub(1).min(self.rows - 1);
                self.x = params[1].saturating_sub(1).min(self.columns - 1);
            }
            b'J' if index == 0 => {
                let cursor = self.y * MAX_COLUMNS + self.x.min(self.columns - 1);
                match params[0] {
                    0 => self.cells[cursor..].fill(b' '),
                    1 => self.cells[..=cursor].fill(b' '),
                    2 => self.cells.fill(b' '),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
