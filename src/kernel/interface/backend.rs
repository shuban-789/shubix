use super::{MAX_COLUMNS, MAX_ROWS, ansi::AnsiSurface, input::Decoder, window::Desktop};
use crate::{spinlock::SpinLock, uart};
use core::fmt::{self, Write};

struct Runtime {
    desktop: Desktop,
    decoder: Decoder,
    active: bool,
}

static DESKTOP: SpinLock<Runtime> = SpinLock::new(Runtime {
    desktop: Desktop::new(),
    decoder: Decoder::new(),
    active: false,
});

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            unsafe {
                uart::uart_putc(byte);
            }
        }
        Ok(())
    }
}

fn render(desktop: &mut Desktop) {
    let mut surface = AnsiSurface::new(Serial);
    desktop.render(&mut surface);
    let r = desktop.bounds();
    let (x, y) = desktop.terminal.cursor();
    // Restore the input position before revealing the cursor after painting.
    surface.finish(r.x + 1 + x, r.y + 1 + y);
}

/// The terminal is the first window client; the shell owns its command loop.
/// Only this thread reads UART, so window controls cannot race shell input.
pub fn run_terminal() {
    // The scheduler is running now. Leave the completed boot report readable
    // for one second before switching to the desktop, without busy-waiting.
    crate::process::sleep(1_000_000);
    {
        let mut runtime = DESKTOP.lock();
        runtime.active = runtime.desktop.init(MAX_COLUMNS, MAX_ROWS);
        let _ = Serial.write_str("\x1b[0m\x1b[2J\x1b[H");
        render(&mut runtime.desktop);
    }
    crate::shell::run();
}

/// Once active, consume console output and paint it within the terminal window.
/// Before startup, console retains its original boot-output path.
pub fn putc(byte: u8) -> bool {
    let mut runtime = DESKTOP.lock();
    if !runtime.active {
        return false;
    }
    runtime.desktop.terminal.putc(byte);
    render(&mut runtime.desktop);
    true
}

pub fn try_read_byte() -> Option<u8> {
    for _ in 0..32 {
        let byte = unsafe { uart::uart_try_getc() }?;
        let mut runtime = DESKTOP.lock();
        if !runtime.active {
            return Some(byte);
        }
        if let Some(key) = runtime.decoder.feed(byte) {
            let result = runtime.desktop.key(key);
            render(&mut runtime.desktop);
            if result.is_some() {
                return result;
            }
        }
    }
    None
}
