#![no_std]
#![no_main]

#[path = "../common/mod.rs"]
mod common;

use core::ptr::{read_volatile, write_volatile};
use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { 
        unsafe { asm!("wfe"); }
    }
}

pub unsafe fn mmio_write(register: u32, data: u32) {
    unsafe {
        write_volatile(register as *mut u32, data);
    }
}

pub unsafe fn mmio_read(register: u32) -> u32 {
    unsafe {
        return read_volatile(register as *const u32);
    }
}

pub unsafe fn delay(count: i32) {
    unsafe {
        asm!(
            "1:",
            "subs {0}, {0}, #1;",
            "bne 1b",
            inout(reg) count => _,
            options(nostack)
        );
    }
}

pub struct Gpio;

impl Gpio {
    pub const GPIO_BASE: u32 = 0x3F200000;

    pub const GPPUD: u32 = (Self::GPIO_BASE + 0x94);
    pub const GPPUDCLK0: u32 = (Self::GPIO_BASE + 0x98);

    pub const UART0_BASE: u32 = 0x3F201000;

    pub const UART0_DR: u32 = (Self::UART0_BASE + 0x00);
    pub const UART0_RSRECR: u32 = (Self::UART0_BASE + 0x04);
    pub const UART0_FR: u32 = (Self::UART0_BASE + 0x18);
    pub const UART0_ILPR: u32 = (Self::UART0_BASE + 0x20);
    pub const UART0_IBRD: u32 = (Self::UART0_BASE + 0x24);
    pub const UART0_FBRD: u32 = (Self::UART0_BASE + 0x28);
    pub const UART0_LCRH: u32 = (Self::UART0_BASE + 0x2C);
    pub const UART0_CR: u32 = (Self::UART0_BASE + 0x30);
    pub const UART0_IFLS: u32 = (Self::UART0_BASE + 0x34);
    pub const UART0_IMSC: u32 = (Self::UART0_BASE + 0x38);
    pub const UART0_RIS: u32 = (Self::UART0_BASE + 0x3C);
    pub const UART0_MIS: u32 = (Self::UART0_BASE + 0x40);
    pub const UART0_ICR: u32 = (Self::UART0_BASE + 0x44);
    pub const UART0_DMACR: u32 = (Self::UART0_BASE + 0x48);
    pub const UART0_ITCR: u32 = (Self::UART0_BASE + 0x80);
    pub const UART0_ITIP: u32 = (Self::UART0_BASE + 0x84);
    pub const UART0_ITOP: u32 = (Self::UART0_BASE + 0x88);
    pub const UART0_TDR: u32 = (Self::UART0_BASE + 0x8C);
}

pub unsafe fn uart_init() {
    unsafe {
        mmio_write(Gpio::UART0_CR, 0x00000000);

        mmio_write(Gpio::GPPUD, 0x00000000);
        delay(150);

        mmio_write(Gpio::GPPUDCLK0, (1 << 14) | (1 << 15));
        delay(150);

        mmio_write(Gpio::GPPUDCLK0, 0x00000000);

        mmio_write(Gpio::UART0_ICR, 0x7FF);

        mmio_write(Gpio::UART0_IBRD, 1);
        mmio_write(Gpio::UART0_FBRD, 40);

        mmio_write(Gpio::UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));

        mmio_write(Gpio::UART0_IMSC, 
            (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
            (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10)
        );

        mmio_write(Gpio::UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
    }
}

pub unsafe fn uart_putc(c: u8) {
    unsafe {
        while mmio_read(Gpio::UART0_FR) & (1 << 5) != 0 {}
        mmio_write(Gpio::UART0_DR, c as u32);
    }
}

pub unsafe fn uart_getc() -> u8 {
    unsafe {
        while mmio_read(Gpio::UART0_FR) & (1 << 4) != 0 {}
        (mmio_read(Gpio::UART0_DR) & 0xFF) as u8
    }
}

pub unsafe fn uart_puts(text: *const u8) {
    unsafe {
        let mut i = 0;
        while *text.add(i) != 0 {
            uart_putc(*text.add(i));
            i += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main(r0: u32, r1: u32, atags: u32) {
    let _ = r0;
    let _ = r1;
    let _ = atags;

    unsafe {
        uart_init();
        uart_puts(b"Shubix\r\n\0".as_ptr());

        loop {
            uart_putc(uart_getc());
            uart_putc(b'\n');
            // main control loop
        }
    }
}
