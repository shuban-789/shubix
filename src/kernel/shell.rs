use crate::{console, process, uart, commands};
use core::str;

fn prompt() -> &'static str {
    // construct some complex prompt later via some external calls
    "> "
}

fn handle<const N: usize>(length: usize, line: [u8; N]) {
    let l: &str = str::from_utf8(&line).expect("Command tokens failed to convert to &str");
    match l {
        "help" => { commands::help() }
        _ => { commands::invalid(line) }
    }
}

pub fn run() {
    let mut line = [0u8; 128];
    let mut len = 0usize;

    crate::kprint!("{}", prompt());
    loop {
        if let Some(byte) = unsafe { uart::uart_try_getc() } {
            if byte == '\n' as u8 || byte == '\r' as u8 || byte == 0x03 { // enter, ^C, etc
                handle(len, line);
                break;
            }
            if byte == 8 || byte == 127 { // handle backspace
                if len > 0 {
                    len -= 1;
                    console::putc(8);
                }
            } else if byte >= b' ' && byte <= b'~' { // printable
                if len < line.len() {
                    line[len] = byte;
                    len += 1;
                    console::putc(byte);
                }
            }
        } else {
            process::sleep(10_000);
        }
    }
}