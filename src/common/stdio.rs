use crate::{uart_getc, uart_putc};

pub unsafe fn getc() -> u8 {
    unsafe { 
        uart_getc() 
    }
}

pub unsafe fn putc(c: u8) {
    unsafe { 
        uart_putc(c); 
    }
}

// simpler puts() api
pub unsafe fn puts(str: *const u8) {
    unsafe {
        let mut i = 0;
        while *str.add(i) != 0 {
            putc(*str.add(i));
            i += 1;
        }
    }
}

// simpler gets() api
pub unsafe fn gets(buf: *mut u8, mut buflen: i32) {
    unsafe {
        let mut i = 0;
        let mut c;
        loop {
            c = getc();
            if c == b'\r' || buflen <= 1 {
                break;
            }
            putc(c);
            *buf.add(i) = c;
            i += 1;
            buflen -= 1;
        }

        putc(b'\n');
        if c == b'\n' {
            *buf.add(i) = 0;
        } else {
            *buf.add((buflen - 1) as usize) = 0;
        }
    }
}
