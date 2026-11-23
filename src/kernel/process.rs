use crate::interrupts::{self, IrqGuard};
use crate::{mem, timer};
use core::ptr::null_mut;

const STACK_PAGES: usize = 4;
const STACK_CANARY: u32 = 0x51ac_cafe;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Ready,
    Blocked,
    Sleeping,
    Exited,
}

#[repr(C)]
pub(crate) struct Process {
    saved_sp: usize,
    stack: *mut u8,
    pub(crate) pid: u32,
    name: [u8; 20],
    state: State,
    wake_at: u64,
    entry: fn(),
    next_queue: *mut Process,
    next_all: *mut Process,
    next_retired: *mut Process,
}

pub(crate) struct Queue {
    head: *mut Process,
    tail: *mut Process,
}

impl Queue {
    pub(crate) const fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }

    pub(crate) unsafe fn push(&mut self, process: *mut Process) {
        unsafe {
            (*process).next_queue = null_mut();
            if self.tail.is_null() {
                self.head = process;
            } else {
                (*self.tail).next_queue = process;
            }
            self.tail = process;
        }
    }

    pub(crate) unsafe fn pop(&mut self) -> *mut Process {
        unsafe {
            let process = self.head;
            if !process.is_null() {
                self.head = (*process).next_queue;
                (*process).next_queue = null_mut();
                if self.head.is_null() {
                    self.tail = null_mut();
                }
            }
            process
        }
    }
}

struct Scheduler {
    current: *mut Process,
    idle: *mut Process,
    all: *mut Process,
    retired: *mut Process,
    ready: Queue,
    next_pid: u32,
    switches: u64,
}

static mut SCHEDULER: Scheduler = Scheduler {
    current: null_mut(),
    idle: null_mut(),
    all: null_mut(),
    retired: null_mut(),
    ready: Queue::new(),
    next_pid: 1,
    switches: 0,
};

unsafe extern "C" {
    fn switch_to_thread(old_sp: *mut usize, new_sp: usize);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    OutOfMemory,
    NotInitialized,
    AlreadyInitialized,
    PidExhausted,
}

pub fn process_init() -> Result<(), ProcessError> {
    let _guard = IrqGuard::new();
    unsafe {
        let scheduler = &mut *(&raw mut SCHEDULER);
        if !scheduler.current.is_null() {
            return Err(ProcessError::AlreadyInitialized);
        }
        let pcb = mem::kmalloc(size_of::<Process>()).cast::<Process>();
        if pcb.is_null() {
            return Err(ProcessError::OutOfMemory);
        }
        let mut name = [0; 20];
        name[..4].copy_from_slice(b"Idle");
        pcb.write(Process {
            saved_sp: 0,
            stack: null_mut(),
            pid: scheduler.next_pid,
            name,
            state: State::Running,
            wake_at: 0,
            entry: idle,
            next_queue: null_mut(),
            next_all: null_mut(),
            next_retired: null_mut(),
        });
        scheduler.next_pid += 1;
        scheduler.current = pcb;
        scheduler.idle = pcb;
        scheduler.all = pcb;
    }
    Ok(())
}

pub fn create_kernel_thread(entry: fn(), name: &str) -> Result<u32, ProcessError> {
    let _guard = IrqGuard::new();
    unsafe {
        let scheduler = &mut *(&raw mut SCHEDULER);
        if scheduler.current.is_null() {
            return Err(ProcessError::NotInitialized);
        }
        let next_pid = scheduler
            .next_pid
            .checked_add(1)
            .ok_or(ProcessError::PidExhausted)?;
        let pcb = mem::kmalloc(size_of::<Process>()).cast::<Process>();
        if pcb.is_null() {
            return Err(ProcessError::OutOfMemory);
        }
        let stack = mem::alloc_pages(STACK_PAGES);
        if stack.is_null() {
            let _ = mem::kfree(pcb.cast());
            return Err(ProcessError::OutOfMemory);
        }
        stack.cast::<u32>().write(STACK_CANARY);
        let saved_sp = stack.add(STACK_PAGES * mem::PAGE_SIZE - 112).cast::<u32>();
        saved_sp.add(27).write(thread_entry as *const () as u32);
        let pid = scheduler.next_pid;
        let mut proc_name = [0; 20];
        let len = name.len().min(19);
        proc_name[..len].copy_from_slice(&name.as_bytes()[..len]);
        pcb.write(Process {
            saved_sp: saved_sp as usize,
            stack,
            pid,
            name: proc_name,
            state: State::Ready,
            wake_at: 0,
            entry,
            next_queue: null_mut(),
            next_all: scheduler.all,
            next_retired: null_mut(),
        });
        scheduler.next_pid = next_pid;
        scheduler.all = pcb;
        scheduler.ready.push(pcb);
        Ok(pid)
    }
}

extern "C" fn thread_entry() -> ! {
    let entry = unsafe {
        reap_retired();
        (*SCHEDULER.current).entry
    };
    unsafe {
        interrupts::enable();
    }
    entry();
    exit()
}

pub fn current_pid() -> u32 {
    let _guard = IrqGuard::new();
    unsafe {
        if SCHEDULER.current.is_null() {
            0
        } else {
            (*SCHEDULER.current).pid
        }
    }
}

pub fn context_switches() -> u64 {
    let _guard = IrqGuard::new();
    unsafe { SCHEDULER.switches }
}

pub(crate) unsafe fn current() -> *mut Process {
    unsafe { SCHEDULER.current }
}

pub fn yield_now() {
    let _guard = IrqGuard::new();
    unsafe {
        schedule();
    }
}

pub fn sleep(microseconds: u32) {
    assert!(interrupts::enabled(), "sleep with IRQs disabled");
    if microseconds == 0 {
        yield_now();
        return;
    }
    let _guard = IrqGuard::new();
    unsafe {
        let current = SCHEDULER.current;
        assert!(
            !current.is_null() && current != SCHEDULER.idle,
            "idle cannot sleep"
        );
        (*current).wake_at = timer::now().wrapping_add(microseconds as u64);
        (*current).state = State::Sleeping;
        schedule();
    }
}

pub(crate) unsafe fn block(queue: *mut Queue) {
    unsafe {
        let current = SCHEDULER.current;
        assert!(
            !current.is_null() && current != SCHEDULER.idle,
            "idle cannot block"
        );
        (*current).state = State::Blocked;
        (*queue).push(current);
        schedule();
    }
}

pub(crate) unsafe fn wake(process: *mut Process) {
    unsafe {
        assert!((*process).state == State::Blocked, "invalid wake");
        (*process).state = State::Ready;
        (*(&raw mut SCHEDULER)).ready.push(process);
    }
}

pub(crate) unsafe fn schedule() {
    let _guard = IrqGuard::new();
    unsafe {
        let scheduler = &raw mut SCHEDULER;
        let old = (*scheduler).current;
        if old.is_null() {
            return;
        }
        reap_retired();
        let now = timer::now();
        let mut process = (*scheduler).all;
        while !process.is_null() {
            if (*process).state == State::Sleeping
                && now.wrapping_sub((*process).wake_at) as i64 >= 0
            {
                (*process).state = State::Ready;
                (*scheduler).ready.push(process);
            }
            process = (*process).next_all;
        }
        if !(*old).stack.is_null() && (*old).stack.cast::<u32>().read() != STACK_CANARY {
            panic!("kernel stack overflow");
        }
        let mut next = (*scheduler).ready.pop();
        if next.is_null() {
            if (*old).state == State::Running {
                return;
            }
            next = (*scheduler).idle;
        }
        if (*old).state == State::Running {
            (*old).state = State::Ready;
            if old != (*scheduler).idle {
                (*scheduler).ready.push(old);
            }
        }
        (*next).state = State::Running;
        (*scheduler).current = next;
        (*scheduler).switches = (*scheduler).switches.wrapping_add(1);
        switch_to_thread(&raw mut (*old).saved_sp, (*next).saved_sp);
        reap_retired();
    }
}

pub fn exit() -> ! {
    let _guard = IrqGuard::new();
    unsafe {
        let scheduler = &raw mut SCHEDULER;
        let old = (*scheduler).current;
        assert!(
            !old.is_null() && old != (*scheduler).idle,
            "idle cannot exit"
        );
        (*old).state = State::Exited;
        let mut link = &raw mut (*scheduler).all;
        while !(*link).is_null() && *link != old {
            link = &raw mut (**link).next_all;
        }
        if *link == old {
            *link = (*old).next_all;
        }
        (*old).next_retired = (*scheduler).retired;
        (*scheduler).retired = old;
        schedule();
    }
    crate::halt()
}

unsafe fn reap_retired() {
    unsafe {
        let scheduler = &raw mut SCHEDULER;
        let mut link = &raw mut (*scheduler).retired;
        while !(*link).is_null() {
            let process = *link;
            if process == (*scheduler).current {
                link = &raw mut (*process).next_retired;
                continue;
            }
            *link = (*process).next_retired;
            let _ = mem::free_pages((*process).stack, STACK_PAGES);
            let _ = mem::kfree(process.cast());
        }
    }
}

pub fn idle() {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}
