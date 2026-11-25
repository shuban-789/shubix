use crate::{console, process, interface, commands};

fn prompt() -> &'static str {
    // construct some complex prompt later via some external calls; further expand apis for fs operations
    "> "
}

fn handle<const N: usize>(length: usize, line: [u8; N]) { // match based off of PATH eventually after fs system is implemented
    let l = core::str::from_utf8(&line[..length]).expect("Command tokens failed to convert to &str");
    let mut tokens = l.split_ascii_whitespace();
    let Some(command) = tokens.next() else {
        return;
    };

    match command {
        "help" => { commands::help() }
        "ls" => { commands::ls(tokens) }
        "cat" => { commands::cat(tokens) }
        _ => { commands::invalid(command) }
    }
}

// todo: make posix compliant somehow; there is some guide somewhere
pub fn run() {
    let mut line = [0u8; 128];
    let mut len = 0usize;

    crate::kprint!("{}", prompt());
    loop {
        if let Some(byte) = interface::try_read_byte() {
            if byte == '\n' as u8 || byte == '\r' as u8 || byte == 0x03 { // enter, ^C, etc
                crate::kprintln!();
                handle(len, line);
                len = 0;
                crate::kprint!("{}", prompt());
                continue;
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
