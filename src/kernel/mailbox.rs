use crate::atag::Memory;
use crate::peripheral::{self, MAILBOX, read, write};
use crate::spinlock::SpinLock;
use core::ptr::{read_volatile, write_volatile};

#[repr(C, align(16))]
struct Message([u32; 64]);

struct Mailbox {
    message: Message,
    poisoned: bool,
}

static MAIL: SpinLock<Mailbox> = SpinLock::new(Mailbox {
    message: Message([0; 64]),
    poisoned: false,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailError {
    InvalidMessage,
    Timeout,
    FirmwareError,
}

pub fn property(words: &mut [u32]) -> Result<(), MailError> {
    if words.len() < 3
        || words.len() > 64
        || words[0] as usize != words.len() * 4
        || words[words.len() - 1] != 0
    {
        return Err(MailError::InvalidMessage);
    }
    let mut mail = MAIL.lock();
    if mail.poisoned {
        return Err(MailError::Timeout);
    }
    let ptr = mail.message.0.as_mut_ptr();
    unsafe {
        for (index, word) in words.iter().enumerate() {
            write_volatile(ptr.add(index), *word);
        }
        peripheral::barrier();
        let request = (ptr as u32 | 0x4000_0000) | 8;
        let start = crate::timer::now();
        while read(MAILBOX + 0x18) & 0x8000_0000 != 0 {
            if crate::timer::now().wrapping_sub(start) > 1_000_000 {
                return Err(MailError::Timeout);
            }
        }
        write(MAILBOX + 0x20, request);
        loop {
            if crate::timer::now().wrapping_sub(start) > 1_000_000 {
                mail.poisoned = true;
                return Err(MailError::Timeout);
            }
            if read(MAILBOX + 0x18) & 0x4000_0000 == 0 && read(MAILBOX) == request {
                break;
            }
        }
        peripheral::barrier();
        for (index, word) in words.iter_mut().enumerate() {
            *word = read_volatile(ptr.add(index));
        }
    }
    if words[1] != 0x8000_0000 {
        return Err(MailError::FirmwareError);
    }
    let mut index = 2;
    while index < words.len() - 1 {
        if index + 3 > words.len() - 1 {
            return Err(MailError::FirmwareError);
        }
        let size = words[index + 1] as usize;
        let response = words[index + 2];
        if size % 4 != 0 || response & 0x8000_0000 == 0 || (response & 0x7fff_ffff) as usize > size
        {
            return Err(MailError::FirmwareError);
        }
        index = index
            .checked_add(3 + size / 4)
            .ok_or(MailError::FirmwareError)?;
    }
    if index != words.len() - 1 {
        return Err(MailError::FirmwareError);
    }
    Ok(())
}

pub fn arm_memory() -> Option<Memory> {
    let mut message = [32, 0, 0x0001_0005, 8, 0, 0, 0, 0];
    property(&mut message).ok()?;
    if message[4] != 0x8000_0008 || message[6] == 0 {
        return None;
    }
    Some(Memory {
        start: message[5],
        size: message[6],
    })
}

pub fn set_uart_clock() -> Result<(), MailError> {
    let mut message = [36, 0, 0x0003_8002, 12, 12, 2, 3_000_000, 0, 0];
    property(&mut message)
}
