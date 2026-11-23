use crate::interrupts::{self, IrqGuard};
use crate::process::{self, Process, Queue};
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;

struct MutexState {
    owner: *mut Process,
    waiting: Queue,
}

pub struct Mutex<T> {
    state: UnsafeCell<MutexState>,
    value: UnsafeCell<T>,
}
unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: UnsafeCell::new(MutexState {
                owner: null_mut(),
                waiting: Queue::new(),
            }),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        assert!(interrupts::enabled(), "mutex lock with IRQs disabled");
        let _irq = IrqGuard::new();
        unsafe {
            let current = process::current();
            assert!(!current.is_null(), "mutex before scheduler initialization");
            let state = self.state.get();
            assert!((*state).owner != current, "recursive mutex lock");
            if (*state).owner.is_null() {
                (*state).owner = current;
            } else {
                process::block(&raw mut (*state).waiting);
                assert!((*state).owner == current, "invalid mutex handoff");
            }
        }
        MutexGuard {
            mutex: self,
            _not_send: PhantomData,
        }
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    _not_send: PhantomData<*mut ()>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        let _irq = IrqGuard::new();
        unsafe {
            let state = self.mutex.state.get();
            assert!((*state).owner == process::current(), "mutex owner mismatch");
            let next = (*state).waiting.pop();
            (*state).owner = next;
            if !next.is_null() {
                process::wake(next);
            }
        }
    }
}
