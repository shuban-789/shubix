use crate::mailbox::{self, MailError};

pub struct Framebuffer {
    pub address: usize,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub rgb: bool,
}

pub fn framebuffer_init() -> Result<Framebuffer, MailError> {
    let mut message = [
        140,
        0,
        0x0004_8003,
        8,
        8,
        640,
        480,
        0x0004_8004,
        8,
        8,
        640,
        480,
        0x0004_8005,
        4,
        4,
        32,
        0x0004_8006,
        4,
        4,
        1,
        0x0004_8009,
        8,
        8,
        0,
        0,
        0x0004_0001,
        8,
        4,
        4096,
        0,
        0x0004_0008,
        4,
        0,
        0,
        0,
    ];
    mailbox::property(&mut message)?;
    let fb = Framebuffer {
        address: (message[28] & 0x3fff_ffff) as usize,
        size: message[29] as usize,
        width: message[5] as usize,
        height: message[6] as usize,
        pitch: message[33] as usize,
        rgb: message[19] == 1,
    };
    if message[15] != 32
        || message[19] > 1
        || fb.address == 0
        || fb.address % 4 != 0
        || fb.width < 8
        || fb.height < 8
        || fb.width > 4096
        || fb.height > 4096
        || fb.pitch < fb.width * 4
        || fb.pitch % 4 != 0
        || fb
            .pitch
            .checked_mul(fb.height)
            .is_none_or(|size| size > fb.size)
        || fb
            .address
            .checked_add(fb.size)
            .is_none_or(|end| end > crate::peripheral::BASE)
    {
        return Err(MailError::FirmwareError);
    }
    Ok(fb)
}
