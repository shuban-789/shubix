use crate::{console, process, uart};
use core::str;

pub fn help() {
    crate::kprintln!("Available:");
    crate::kprint!("help -> list available commands");
}

pub fn invalid<const N: usize>(length: usize, line: [u8; N]) {
    if length != 0 {
        let l: &str = str::from_utf8(&line).expect("Command tokens failed to convert to &str");
        crate::kprintln!("Unknown: {}", l);
    }
}