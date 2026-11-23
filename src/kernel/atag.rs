pub const ATAG_NONE: u32 = 0;
pub const ATAG_MEM: u32 = 0x5441_0002;

#[repr(C)]
pub struct Atag {
    pub size: u32,
    pub tag: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memory {
    pub size: u32,
    pub start: u32,
}

pub unsafe fn get_memory(atags: *const Atag) -> Option<Memory> {
    if atags.is_null() || !(atags as usize).is_multiple_of(4) {
        return None;
    }

    if atags as usize >= crate::peripheral::BASE - 8 {
        return None;
    }
    let mut words = atags.cast::<u32>();
    let limit = (words as usize)
        .checked_add(64 * 1024)?
        .min(crate::peripheral::BASE);
    loop {
        unsafe {
            let tag = words.add(1).read();
            if tag == ATAG_NONE {
                return None;
            }
            let len = words.read() as usize;
            if len < 2 || len > 4096 || tag & 0xffff_0000 != 0x5441_0000 {
                return None;
            }
            if tag == ATAG_MEM {
                if len < 4 || (words as usize).checked_add(16)? > limit {
                    return None;
                }
                return Some(Memory {
                    size: words.add(2).read(),
                    start: words.add(3).read(),
                });
            }
            let bytes = len.checked_mul(size_of::<u32>())?;
            let next = (words as usize).checked_add(bytes)?;
            if next.checked_add(8)? > limit {
                return None;
            }
            words = next as *const u32;
        }
    }
}

pub unsafe fn get_mem_size(atags: *const Atag) -> Option<u32> {
    unsafe { get_memory(atags).map(|memory| memory.size) }
}
