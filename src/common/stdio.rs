pub fn getc() -> u8 {
    loop {
        if let Some(byte) = unsafe { crate::uart::uart_try_getc() } {
            return byte;
        }
        if crate::interrupts::enabled() && crate::process::current_pid() > 1 {
            crate::process::sleep(10_000);
        } else {
            core::hint::spin_loop();
        }
    }
}

pub fn putc(byte: u8) {
    crate::console::putc(byte);
}

pub unsafe fn puts(text: *const u8) {
    unsafe {
        let mut ptr = text;
        while ptr.read() != 0 {
            putc(ptr.read());
            ptr = ptr.add(1);
        }
    }
}

pub unsafe fn gets(buf: *mut u8, buflen: usize) {
    if buflen == 0 {
        return;
    }
    unsafe {
        let mut index = 0;
        loop {
            let byte = getc();
            if byte == b'\r' || byte == b'\n' {
                break;
            }
            if byte == 8 || byte == 127 {
                if index > 0 {
                    index -= 1;
                    putc(8);
                }
            } else if index < buflen - 1 {
                buf.add(index).write(byte);
                index += 1;
                putc(byte);
            }
        }
        buf.add(index).write(0);
        putc(b'\n');
    }
}
