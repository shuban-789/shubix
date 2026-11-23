use crate::peripheral::{self, GPIO, UART0, read, write};

pub unsafe fn uart_init() {
    unsafe {
        write(UART0 + 0x30, 0);
        let pins = read(GPIO + 0x04);
        write(
            GPIO + 0x04,
            (pins & !((7 << 12) | (7 << 15))) | (4 << 12) | (4 << 15),
        );
        write(GPIO + 0x94, 0);
        peripheral::delay(150);
        write(GPIO + 0x98, (1 << 14) | (1 << 15));
        peripheral::delay(150);
        write(GPIO + 0x98, 0);
        write(UART0 + 0x44, 0x7ff);
        write(UART0 + 0x38, 0);
        write(UART0 + 0x24, 1);
        write(UART0 + 0x28, 40);
        write(UART0 + 0x2c, (1 << 4) | (1 << 5) | (1 << 6));
        write(UART0 + 0x30, (1 << 0) | (1 << 8) | (1 << 9));
        peripheral::barrier();
    }
}

pub unsafe fn uart_putc(byte: u8) {
    unsafe {
        while read(UART0 + 0x18) & (1 << 5) != 0 {
            core::hint::spin_loop();
        }
        write(UART0, byte as u32);
    }
}

pub unsafe fn uart_try_getc() -> Option<u8> {
    unsafe {
        if read(UART0 + 0x18) & (1 << 4) != 0 {
            None
        } else {
            Some(read(UART0) as u8)
        }
    }
}

pub unsafe fn uart_getc() -> u8 {
    loop {
        if let Some(byte) = unsafe { uart_try_getc() } {
            return byte;
        }
        core::hint::spin_loop();
    }
}

pub unsafe fn uart_puts(text: *const u8) {
    unsafe {
        let mut ptr = text;
        while ptr.read() != 0 {
            uart_putc(ptr.read());
            ptr = ptr.add(1);
        }
    }
}
