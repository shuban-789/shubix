use crate::atag::{self, Atag, Memory};
use crate::interrupts::IrqGuard;
use core::ptr::{addr_of, null_mut};

pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024;
const NONE: usize = usize::MAX;
const FREE: u32 = 0;
const RESERVED: u32 = 1;
const ALLOCATED: u32 = 2;

unsafe extern "C" {
    static __end: u8;
}

struct Page {
    next: usize,
    state: u32,
}

struct PageAllocator {
    base: usize,
    count: usize,
    reserved: usize,
    pages: *mut Page,
    head: usize,
    free_count: usize,
}

static mut PAGES: PageAllocator = PageAllocator {
    base: 0,
    count: 0,
    reserved: 0,
    pages: null_mut(),
    head: NONE,
    free_count: 0,
};
static mut HEAP: *mut Segment = null_mut();

#[repr(C, align(16))]
struct Segment {
    next: *mut Segment,
    prev: *mut Segment,
    size: usize,
    allocated: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemError {
    MissingMemoryTag,
    InvalidMemoryRange,
    InsufficientMemory,
    AlreadyInitialized,
    InvalidPage,
    InvalidAllocation,
}

#[derive(Clone, Copy)]
pub struct MemoryStats {
    pub total_pages: usize,
    pub reserved_pages: usize,
    pub free_pages: usize,
    pub heap_free: usize,
    pub heap_largest: usize,
}

fn align_up(value: usize, alignment: usize) -> Option<usize> {
    value
        .checked_add(alignment - 1)
        .map(|v| v & !(alignment - 1))
}

pub unsafe fn mem_init(atags: *const Atag) -> Result<(), MemError> {
    let memory = unsafe { atag::get_memory(atags) }
        .or_else(crate::mailbox::arm_memory)
        .ok_or(MemError::MissingMemoryTag)?;
    unsafe { init_region(memory, addr_of!(__end) as usize) }
}

unsafe fn init_region(memory: Memory, kernel_end: usize) -> Result<(), MemError> {
    let _guard = IrqGuard::new();
    unsafe {
        let allocator = &mut *(&raw mut PAGES);
        if !allocator.pages.is_null() {
            return Err(MemError::AlreadyInitialized);
        }
        let base = memory.start as usize;
        let count = memory.size as usize / PAGE_SIZE;
        let end = base
            .checked_add(count * PAGE_SIZE)
            .ok_or(MemError::InvalidMemoryRange)?;
        if base != 0 || end > crate::peripheral::BASE || kernel_end <= base || kernel_end >= end {
            return Err(MemError::InvalidMemoryRange);
        }
        let metadata =
            align_up(kernel_end, align_of::<Page>()).ok_or(MemError::InvalidMemoryRange)?;
        let metadata_size = count
            .checked_mul(size_of::<Page>())
            .ok_or(MemError::InvalidMemoryRange)?;
        let heap_start = metadata
            .checked_add(metadata_size)
            .and_then(|p| align_up(p, PAGE_SIZE))
            .ok_or(MemError::InvalidMemoryRange)?;
        let reserved_end = heap_start
            .checked_add(KERNEL_HEAP_SIZE)
            .ok_or(MemError::InvalidMemoryRange)?;
        if reserved_end >= end {
            return Err(MemError::InsufficientMemory);
        }
        let reserved = (reserved_end - base) / PAGE_SIZE;
        let pages = metadata as *mut Page;
        for index in 0..count {
            pages.add(index).write(Page {
                next: if index >= reserved && index + 1 < count {
                    index + 1
                } else {
                    NONE
                },
                state: if index < reserved { RESERVED } else { FREE },
            });
        }
        *allocator = PageAllocator {
            base,
            count,
            reserved,
            pages,
            head: reserved,
            free_count: count - reserved,
        };
        HEAP = heap_start as *mut Segment;
        HEAP.write(Segment {
            next: null_mut(),
            prev: null_mut(),
            size: KERNEL_HEAP_SIZE,
            allocated: 0,
        });
    }
    Ok(())
}

pub unsafe fn alloc_page() -> *mut u8 {
    unsafe { alloc_pages(1) }
}

pub unsafe fn alloc_pages(count: usize) -> *mut u8 {
    let _guard = IrqGuard::new();
    unsafe {
        let a = &mut *(&raw mut PAGES);
        if count == 0 || count > a.free_count {
            return null_mut();
        }
        let start;
        if count == 1 {
            start = a.head;
            a.head = (*a.pages.add(start)).next;
        } else {
            let mut run = 0;
            let mut found = NONE;
            for index in a.reserved..a.count {
                if (*a.pages.add(index)).state == FREE {
                    run += 1;
                } else {
                    run = 0;
                }
                if run == count {
                    found = index + 1 - count;
                    break;
                }
            }
            if found == NONE {
                return null_mut();
            }
            start = found;
            let mut link = &raw mut a.head;
            while *link != NONE {
                let index = *link;
                let page = a.pages.add(index);
                if index >= start && index < start + count {
                    *link = (*page).next;
                } else {
                    link = &raw mut (*page).next;
                }
            }
        }
        for index in start..start + count {
            (*a.pages.add(index)).state = ALLOCATED;
            (*a.pages.add(index)).next = NONE;
        }
        a.free_count -= count;
        let ptr = (a.base + start * PAGE_SIZE) as *mut u8;
        ptr.write_bytes(0, count * PAGE_SIZE);
        ptr
    }
}

pub unsafe fn free_page(ptr: *mut u8) -> Result<(), MemError> {
    unsafe { free_pages(ptr, 1) }
}

pub unsafe fn free_pages(ptr: *mut u8, count: usize) -> Result<(), MemError> {
    let _guard = IrqGuard::new();
    unsafe {
        let a = &mut *(&raw mut PAGES);
        let address = ptr as usize;
        if count == 0 || a.pages.is_null() || address < a.base || !address.is_multiple_of(PAGE_SIZE)
        {
            return Err(MemError::InvalidPage);
        }
        let start = (address - a.base) / PAGE_SIZE;
        let end = start.checked_add(count).ok_or(MemError::InvalidPage)?;
        if start < a.reserved || end > a.count {
            return Err(MemError::InvalidPage);
        }
        for index in start..end {
            if (*a.pages.add(index)).state != ALLOCATED {
                return Err(MemError::InvalidPage);
            }
        }
        for index in (start..end).rev() {
            let page = &mut *a.pages.add(index);
            page.state = FREE;
            page.next = a.head;
            a.head = index;
        }
        a.free_count += count;
    }
    Ok(())
}

pub unsafe fn kmalloc(bytes: usize) -> *mut u8 {
    if bytes == 0 {
        return null_mut();
    }
    let Some(size) = bytes
        .checked_add(size_of::<Segment>())
        .and_then(|n| align_up(n, 16))
    else {
        return null_mut();
    };
    let _guard = IrqGuard::new();
    unsafe {
        let mut best: *mut Segment = null_mut();
        let mut current = HEAP;
        while !current.is_null() {
            if (*current).allocated == 0
                && (*current).size >= size
                && (best.is_null() || (*current).size < (*best).size)
            {
                best = current;
            }
            current = (*current).next;
        }
        if best.is_null() {
            return null_mut();
        }
        let remaining = (*best).size - size;
        if remaining >= size_of::<Segment>() + 16 {
            let tail = (best as *mut u8).add(size).cast::<Segment>();
            tail.write(Segment {
                next: (*best).next,
                prev: best,
                size: remaining,
                allocated: 0,
            });
            if !(*tail).next.is_null() {
                (*(*tail).next).prev = tail;
            }
            (*best).next = tail;
            (*best).size = size;
        }
        (*best).allocated = 1;
        best.add(1).cast()
    }
}

pub unsafe fn kfree(ptr: *mut u8) -> Result<(), MemError> {
    if ptr.is_null() {
        return Ok(());
    }
    let _guard = IrqGuard::new();
    unsafe {
        let mut segment = HEAP;
        while !segment.is_null() && segment.add(1).cast::<u8>() != ptr {
            segment = (*segment).next;
        }
        if segment.is_null() || (*segment).allocated == 0 {
            return Err(MemError::InvalidAllocation);
        }
        (*segment).allocated = 0;
        if !(*segment).prev.is_null() && (*(*segment).prev).allocated == 0 {
            segment = (*segment).prev;
            merge_next(segment);
        }
        if !(*segment).next.is_null() && (*(*segment).next).allocated == 0 {
            merge_next(segment);
        }
    }
    Ok(())
}

unsafe fn merge_next(segment: *mut Segment) {
    unsafe {
        let next = (*segment).next;
        (*segment).size += (*next).size;
        (*segment).next = (*next).next;
        if !(*segment).next.is_null() {
            (*(*segment).next).prev = segment;
        }
    }
}

pub fn stats() -> MemoryStats {
    let _guard = IrqGuard::new();
    unsafe {
        let a = &*(&raw const PAGES);
        let mut stats = MemoryStats {
            total_pages: a.count,
            reserved_pages: a.reserved,
            free_pages: a.free_count,
            heap_free: 0,
            heap_largest: 0,
        };
        let mut segment = HEAP;
        while !segment.is_null() {
            if (*segment).allocated == 0 {
                let size = (*segment).size - size_of::<Segment>();
                stats.heap_free += size;
                stats.heap_largest = stats.heap_largest.max(size);
            }
            segment = (*segment).next;
        }
        stats
    }
}
