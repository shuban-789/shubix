pub const RESIZE_KEY: u8 = 0x07;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Byte(u8),
    Up,
    Down,
    Left,
    Right,
    Resize,
    Move,
}

#[derive(Clone, Copy)]
enum State {
    Ground,
    Escape,
    Csi,
    Ss3,
}

pub struct Decoder {
    state: State,
}

impl Decoder {
    pub const fn new() -> Self {
        Self {
            state: State::Ground,
        }
    }

    pub fn feed(&mut self, byte: u8) -> Option<Key> {
        if byte == RESIZE_KEY {
            self.state = State::Ground;
            return Some(Key::Resize);
        }
        if byte == 0x1b {
            self.state = State::Escape;
            return None;
        }
        match self.state {
            State::Ground => Some(Key::Byte(byte)),
            State::Escape => {
                // Alt prefixes Ctrl+Y with Escape on the serial terminal.
                if byte == 0x19 {
                    self.state = State::Ground;
                    return Some(Key::Move);
                }
                self.state = match byte {
                    b'[' => State::Csi,
                    b'O' => State::Ss3,
                    _ => State::Ground,
                };
                if matches!(self.state, State::Ground) {
                    Some(Key::Byte(byte))
                } else {
                    None
                }
            }
            State::Csi | State::Ss3 => {
                if (0x20..=0x3f).contains(&byte) {
                    return None;
                }
                self.state = State::Ground;
                match byte {
                    b'A' => Some(Key::Up),
                    b'B' => Some(Key::Down),
                    b'C' => Some(Key::Right),
                    b'D' => Some(Key::Left),
                    0..=0x1f => Some(Key::Byte(byte)),
                    _ => None,
                }
            }
        }
    }
}
