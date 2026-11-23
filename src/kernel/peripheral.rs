use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

pub const BASE: usize = 0x3f00_0000;
pub const TIMER: usize = BASE + 0x3000;
pub const INTERRUPTS: usize = BASE + 0xb200;
pub const MAILBOX: usize = BASE + 0xb880;
pub const GPIO: usize = BASE + 0x20_0000;
pub const UART0: usize = BASE + 0x20_1000;

#[inline]
pub unsafe fn read(address: usize) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}

#[inline]
pub unsafe fn write(address: usize, value: u32) {
    unsafe { write_volatile(address as *mut u32, value) }
}

#[inline]
pub fn barrier() {
    unsafe {
        asm!("dsb sy", options(nostack, preserves_flags));
    }
}

pub fn delay(mut cycles: u32) {
    while cycles != 0 {
        unsafe {
            asm!("nop", options(nomem, nostack, preserves_flags));
        }
        cycles -= 1;
    }
}
