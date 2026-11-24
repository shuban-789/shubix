#![no_std]
#![no_main]

pub mod atag;
#[path = "../common/mod.rs"]
pub mod common;
pub mod console;
mod font;
pub mod framebuffer;
pub mod gpu;
pub mod interrupts;
pub mod mailbox;
pub mod mem;
pub mod mutex;
pub mod peripheral;
pub mod process;
pub mod spinlock;
pub mod timer;
pub mod uart;
pub mod shell;
pub mod commands;
pub mod windowing;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

pub use uart::{uart_getc, uart_putc};
global_asm!(include_str!("context.S"), options(raw));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        interrupts::disable();
    }
    console::emergency(format_args!("\nKernel panic: {}\n", info));
    halt()
}

pub fn halt() -> ! {
    unsafe {
        interrupts::disable();
    }
    loop {
        unsafe {
            asm!("wfi", options(nomem, nostack));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main(_r0: u32, _r1: u32, atags: u32) -> ! {
    unsafe {
        uart::uart_init();
        if mailbox::set_uart_clock().is_ok() {
            uart::uart_init();
        }
        console::clear();
        kprintln!("Shubix");
        mem::mem_init(atags as *const atag::Atag).expect("RAM initialization failed");
        let memory = mem::stats();
        kprintln!(
            "Memory: {} MiB, {} free pages, {} KiB heap",
            memory.total_pages * mem::PAGE_SIZE / (1024 * 1024),
            memory.free_pages,
            mem::KERNEL_HEAP_SIZE / 1024
        );
        kprintln!("Console: UART, ANSI text, 80x24");
        interrupts::interrupts_init();
        process::process_init().expect("scheduler initialization failed");
        process::create_kernel_thread(windowing::run_terminal, "Terminal")
            .expect("console thread allocation failed");
        timer::timer_init();
        kprintln!("Scheduler ready: 10 ms quantum");
        kprintln!();
        interrupts::enable();
        process::idle();
    }
    halt()
}
