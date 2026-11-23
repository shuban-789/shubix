use core::ffi::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dest: *mut c_void,
    src: *const c_void,
    bytes: usize,
) -> *mut c_void {
    unsafe {
        let d = dest.cast::<u8>();
        let s = src.cast::<u8>();
        for index in 0..bytes {
            d.add(index).write_volatile(s.add(index).read_volatile());
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut c_void, value: i32, bytes: usize) -> *mut c_void {
    unsafe {
        let d = dest.cast::<u8>();
        for index in 0..bytes {
            d.add(index).write_volatile(value as u8);
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(
    dest: *mut c_void,
    src: *const c_void,
    bytes: usize,
) -> *mut c_void {
    unsafe {
        if (dest as usize) <= src as usize {
            return memcpy(dest, src, bytes);
        }
        let d = dest.cast::<u8>();
        let s = src.cast::<u8>();
        for index in (0..bytes).rev() {
            d.add(index).write_volatile(s.add(index).read_volatile());
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(left: *const c_void, right: *const c_void, bytes: usize) -> i32 {
    unsafe {
        for index in 0..bytes {
            let a = left.cast::<u8>().add(index).read_volatile();
            let b = right.cast::<u8>().add(index).read_volatile();
            if a != b {
                return a as i32 - b as i32;
            }
        }
    }
    0
}

pub unsafe fn bzero(dest: *mut c_void, bytes: usize) {
    unsafe {
        memset(dest, 0, bytes);
    }
}

pub fn itoa(value: i32, buffer: &mut [u8; 12]) -> &str {
    let negative = value < 0;
    let mut magnitude = value.unsigned_abs();
    let mut index = buffer.len();
    loop {
        index -= 1;
        buffer[index] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if negative {
        index -= 1;
        buffer[index] = b'-';
    }
    unsafe { core::str::from_utf8_unchecked(&buffer[index..]) }
}
