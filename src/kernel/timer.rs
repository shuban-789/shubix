use crate::interrupts::{self, IrqGuard};
use crate::peripheral::{self, TIMER, read, write};

pub const QUANTUM_US: u32 = 10_000;
static mut TICKS: u64 = 0;
static mut RESCHEDULE: bool = false;

pub fn now() -> u64 {
    unsafe {
        loop {
            let high = read(TIMER + 0x08);
            let low = read(TIMER + 0x04);
            if high == read(TIMER + 0x08) {
                return ((high as u64) << 32) | low as u64;
            }
        }
    }
}

pub fn ticks() -> u64 {
    let _guard = IrqGuard::new();
    unsafe { TICKS }
}

pub fn udelay(microseconds: u32) {
    let start = now();
    while now().wrapping_sub(start) < microseconds as u64 {
        core::hint::spin_loop();
    }
}

pub unsafe fn timer_set(microseconds: u32) {
    unsafe {
        let deadline = read(TIMER + 0x04).wrapping_add(microseconds.max(1));
        write(TIMER + 0x10, deadline);
        peripheral::barrier();
    }
}

unsafe fn clear() {
    unsafe {
        write(TIMER, 1 << 1);
        peripheral::barrier();
    }
}

unsafe fn tick() {
    unsafe {
        TICKS = TICKS.wrapping_add(1);
        RESCHEDULE = true;
        timer_set(QUANTUM_US);
    }
}

pub unsafe fn timer_init() {
    unsafe {
        clear();
        let _ = interrupts::register_irq_handler(1, tick, clear);
        timer_set(QUANTUM_US);
    }
}

pub(crate) unsafe fn take_reschedule() -> bool {
    unsafe {
        let pending = RESCHEDULE;
        RESCHEDULE = false;
        pending
    }
}
