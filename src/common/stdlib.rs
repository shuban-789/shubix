use core::ffi::c_void;

pub unsafe fn memcpy(dest: *mut c_void, src: *mut c_void, mut bytes: i32) {
    unsafe {
        let mut d = dest.cast::<u8>();
        let mut s = src.cast::<u8>();
        while bytes != 0 {
            bytes -= 1;
            *d = *s;
            d = d.add(1);
            s = s.add(1);
        }
    }
}

pub unsafe fn bzero(dest: *mut c_void, mut bytes: i32) {
    unsafe {
        let mut d = dest.cast::<u8>();
        while bytes != 0 {
            bytes -= 1;
            *d = 0;
            d = d.add(1);
        }
    }
}

pub unsafe fn itoa(mut i: i32) -> *mut u8 {
    static mut INTBUF: [u8; 12] = [0; 12];
    unsafe {
        let intbuf = (&raw mut INTBUF).cast::<u8>();
        let mut j: i32 = 0;
        let mut isneg = 0;

        if i == 0 {
            *intbuf = b'0';
            *intbuf.add(1) = 0;
            return intbuf;
        }

        if i < 0 {
            isneg = 1;
            i = -i;
        }

        while i != 0 {
            *intbuf.add(j as usize) = b'0' + (i % 10) as u8;
            j += 1;
            i /= 10;
        }

        if isneg != 0 {
            *intbuf.add(j as usize) = b'-';
            j += 1;
        }

        *intbuf.add(j as usize) = 0;
        j -= 1;
        i = 0;
        while i < j {
            isneg = *intbuf.add(i as usize) as i32;
            *intbuf.add(i as usize) = *intbuf.add(j as usize);
            *intbuf.add(j as usize) = isneg as u8;
            i += 1;
            j -= 1;
        }

        intbuf
    }
}
