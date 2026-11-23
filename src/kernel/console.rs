use crate::spinlock::SpinLock;
use core::fmt::{self, Write};

static OUTPUT: SpinLock<()> = SpinLock::new(());
struct Writer;

impl Write for Writer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            emit(byte);
        }
        Ok(())
    }
}

fn emit(byte: u8) {
    unsafe {
        if byte == 8 || byte == 127 {
            crate::uart::uart_putc(8);
            crate::uart::uart_putc(b' ');
            crate::uart::uart_putc(8);
        } else {
            if byte == b'\n' {
                crate::uart::uart_putc(b'\r');
            }
            crate::uart::uart_putc(byte);
        }
    }
    crate::gpu::gpu_putc(byte);
}

pub fn print(args: fmt::Arguments<'_>) {
    let _guard = OUTPUT.lock();
    let _ = Writer.write_fmt(args);
}

pub fn putc(byte: u8) {
    let _guard = OUTPUT.lock();
    emit(byte);
}

pub fn emergency(args: fmt::Arguments<'_>) {
    struct Emergency;
    impl Write for Emergency {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            for byte in text.bytes() {
                unsafe {
                    if byte == b'\n' {
                        crate::uart::uart_putc(b'\r');
                    }
                    crate::uart::uart_putc(byte);
                }
            }
            Ok(())
        }
    }
    let _ = Emergency.write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => { $crate::console::print(core::format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => { $crate::kprint!("{}\n", core::format_args!($($arg)*)) };
}
