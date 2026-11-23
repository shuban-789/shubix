use crate::font::FONT;
use crate::framebuffer::{self, Framebuffer};
use crate::mailbox::MailError;
use crate::spinlock::SpinLock;

#[derive(Clone, Copy)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

struct Screen {
    framebuffer: Option<Framebuffer>,
    x: usize,
    y: usize,
}
static SCREEN: SpinLock<Screen> = SpinLock::new(Screen {
    framebuffer: None,
    x: 0,
    y: 0,
});

pub fn gpu_init() -> Result<(), MailError> {
    let fb = framebuffer::framebuffer_init()?;
    let mut screen = SCREEN.lock();
    screen.framebuffer = Some(fb);
    screen.clear();
    Ok(())
}

pub fn gpu_putc(byte: u8) {
    SCREEN.lock().putc(byte);
}
pub fn clear() {
    SCREEN.lock().clear();
}
pub fn write_pixel(x: usize, y: usize, pixel: Pixel) {
    SCREEN.lock().pixel(x, y, pixel);
}

impl Screen {
    fn pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        let Some(fb) = &self.framebuffer else {
            return;
        };
        if x >= fb.width || y >= fb.height {
            return;
        }
        let (first, last) = if fb.rgb {
            (pixel.red, pixel.blue)
        } else {
            (pixel.blue, pixel.red)
        };
        let value =
            0xff00_0000 | ((last as u32) << 16) | ((pixel.green as u32) << 8) | first as u32;
        unsafe {
            ((fb.address + y * fb.pitch + x * 4) as *mut u32).write_volatile(value);
        }
    }

    fn clear(&mut self) {
        if let Some(fb) = &self.framebuffer {
            for offset in (0..fb.pitch * fb.height).step_by(4) {
                unsafe {
                    ((fb.address + offset) as *mut u32).write_volatile(0xff00_0000);
                }
            }
        }
        self.x = 0;
        self.y = 0;
    }

    fn scroll(&mut self) {
        let Some(fb) = &self.framebuffer else {
            return;
        };
        let rows = fb.height / 8;
        if self.y < rows {
            return;
        }
        let line = fb.pitch * 8;
        let used = line * rows;
        unsafe {
            for offset in (0..used - line).step_by(4) {
                let src = (fb.address + line + offset) as *const u32;
                let dst = (fb.address + offset) as *mut u32;
                dst.write_volatile(src.read_volatile());
            }
            for offset in (used - line..used).step_by(4) {
                ((fb.address + offset) as *mut u32).write_volatile(0xff00_0000);
            }
        }
        self.y = rows - 1;
    }

    fn putc(&mut self, byte: u8) {
        let Some(fb) = &self.framebuffer else {
            return;
        };
        let columns = fb.width / 8;
        match byte {
            b'\r' => {
                self.x = 0;
                return;
            }
            b'\n' => {
                self.x = 0;
                self.y += 1;
                self.scroll();
                return;
            }
            b'\t' => {
                for _ in 0..4 - self.x % 4 {
                    self.putc(b' ');
                }
                return;
            }
            8 | 127 => {
                if self.x > 0 {
                    self.x -= 1;
                } else if self.y > 0 {
                    self.y -= 1;
                    self.x = columns - 1;
                } else {
                    return;
                }
                self.draw(b' ');
                return;
            }
            0..=31 => return,
            _ => {}
        }
        self.scroll();
        self.draw(if byte < 128 { byte } else { b'?' });
        self.x += 1;
        if self.x >= columns {
            self.x = 0;
            self.y += 1;
        }
    }

    fn draw(&mut self, byte: u8) {
        for (y, bits) in FONT[byte as usize].iter().enumerate() {
            for x in 0..8 {
                let level = if bits & (1 << x) != 0 { 255 } else { 0 };
                self.pixel(
                    self.x * 8 + x,
                    self.y * 8 + y,
                    Pixel {
                        red: level,
                        green: level,
                        blue: level,
                    },
                );
            }
        }
    }
}
