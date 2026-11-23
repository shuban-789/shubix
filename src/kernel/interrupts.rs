use crate::peripheral::{self, INTERRUPTS, read, write};
use core::arch::asm;
use core::marker::PhantomData;

pub type Handler = unsafe fn();
static mut HANDLERS: [Option<Handler>; 72] = [None; 72];
static mut CLEARERS: [Option<Handler>; 72] = [None; 72];

pub struct IrqGuard {
    enabled: bool,
    _not_send: PhantomData<*mut ()>,
}

impl IrqGuard {
    pub fn new() -> Self {
        let cpsr: u32;
        unsafe {
            asm!("mrs {0}, cpsr", "cpsid i", out(reg) cpsr, options(nostack, preserves_flags));
        }
        Self {
            enabled: cpsr & 0x80 == 0,
            _not_send: PhantomData,
        }
    }
}

impl Drop for IrqGuard {
    fn drop(&mut self) {
        if self.enabled {
            unsafe {
                enable();
            }
        }
    }
}

pub fn enabled() -> bool {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr, options(nomem, nostack, preserves_flags));
    }
    cpsr & 0x80 == 0
}

pub unsafe fn enable() {
    unsafe {
        asm!("cpsie i", options(nostack, preserves_flags));
    }
}

pub unsafe fn disable() {
    unsafe {
        asm!("cpsid i", options(nostack, preserves_flags));
    }
}

pub unsafe fn interrupts_init() {
    unsafe {
        disable();
        write(INTERRUPTS + 0x1c, u32::MAX);
        write(INTERRUPTS + 0x20, u32::MAX);
        write(INTERRUPTS + 0x24, u32::MAX);
        write(INTERRUPTS + 0x0c, 0);
        write(0x4000_000c, 0);
        peripheral::barrier();
    }
}

pub unsafe fn register_irq_handler(
    irq: usize,
    handler: Handler,
    clearer: Handler,
) -> Result<(), ()> {
    if irq >= 72 {
        return Err(());
    }
    let _guard = IrqGuard::new();
    unsafe {
        (&raw mut HANDLERS)
            .cast::<Option<Handler>>()
            .add(irq)
            .write(Some(handler));
        (&raw mut CLEARERS)
            .cast::<Option<Handler>>()
            .add(irq)
            .write(Some(clearer));
        let (register, bit) = if irq < 32 {
            (0x10, irq)
        } else if irq < 64 {
            (0x14, irq - 32)
        } else {
            (0x18, irq - 64)
        };
        write(INTERRUPTS + register, 1 << bit);
        peripheral::barrier();
    }
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn irq_handler() {
    unsafe {
        let pending = [
            read(INTERRUPTS + 0x04),
            read(INTERRUPTS + 0x08),
            read(INTERRUPTS) & 0xff,
        ];
        for (bank, mut bits) in pending.into_iter().enumerate() {
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let index = bank * 32 + bit;
                let handler = (&raw const HANDLERS)
                    .cast::<Option<Handler>>()
                    .add(index)
                    .read();
                let clearer = (&raw const CLEARERS)
                    .cast::<Option<Handler>>()
                    .add(index)
                    .read();
                if let (Some(handler), Some(clearer)) = (handler, clearer) {
                    clearer();
                    handler();
                } else {
                    write(INTERRUPTS + [0x1c, 0x20, 0x24][bank], 1 << bit);
                }
            }
        }
        if crate::timer::take_reschedule() {
            crate::process::schedule();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exception_handler(kind: u32, pc: u32, status: u32, address: u32) -> ! {
    crate::console::emergency(format_args!(
        "\nException {} at {:#010x}, status {:#010x}, address {:#010x}\n",
        kind, pc, status, address
    ));
    crate::halt()
}
