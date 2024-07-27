use core::{
    arch::asm,
    cell::{Cell, UnsafeCell},
    sync::atomic::{compiler_fence, Ordering},
};

use avr_device::interrupt::{CriticalSection, Mutex};

/**
This provides a way to access a static mutex that would otherwise be disallowed.
It functions like avr_device::interrupt::free except that it doesn't actually
block interrupts while the function is running.

This should not be used unless you are sure that an interrupt will not happen
or being interrupted while accessing the mutex will not cause a race condition.
*/
#[inline(always)]
pub fn unsafe_access_mutex<F, R>(f: F) -> R
where
    F: FnOnce(CriticalSection) -> R,
{
    compiler_fence(Ordering::SeqCst);

    let r = f(unsafe { CriticalSection::new() });

    compiler_fence(Ordering::SeqCst);

    r
}

/**
A way to access a mutex if you are sure that interrupts are already disabled, e.g.
if you are calling from inside an interrupt. In debug mode, that assertion will be
checked, but if debug_assertions are disabled then it is unchecked
*/
#[inline(always)]
#[allow(unreachable_code)]
pub fn assert_interrupts_disabled<F, R>(f: F) -> R
where
    F: FnOnce(CriticalSection) -> R,
{
    compiler_fence(Ordering::SeqCst);

    if cfg!(debug_assertions) {
        let sreg: u8;
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "avr")] {
                // Store current state
                unsafe {
                    asm!(
                        "in {sreg}, 0x3F",
                        sreg = out(reg) sreg,
                    )
                };
            } else {
                let _ = sreg;
                unimplemented!()
            }
        }

        let interrupts_enabled = sreg & 0x80 != 0;
        debug_assert!(!interrupts_enabled, "Interrupts were not disabled");
    }

    let r = f(unsafe { CriticalSection::new() });

    compiler_fence(Ordering::SeqCst);

    r
}

pub trait Borrowable {
    type Inner;
    /**
    A way to get the contents of a Mutex inside an UnsafeCell. Sometimes
    UnsafeCell is needed if you want to be able modify a type behind a mutex
    which does not implement Copy, either because it is large or contains
    unique resource handles.
    */
    fn get_inner<'cs>(&self, cs: CriticalSection<'cs>) -> &'cs Self::Inner;
    fn get_inner_mut<'cs>(&self, cs: CriticalSection<'cs>) -> &'cs mut Self::Inner;
}

impl<T> Borrowable for Mutex<UnsafeCell<T>> {
    type Inner = T;
    fn get_inner<'cs>(&self, cs: CriticalSection<'cs>) -> &'cs Self::Inner {
        let ptr = self.borrow(cs).get();
        let inner_ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        inner_ref
    }
    fn get_inner_mut<'cs>(&self, cs: CriticalSection<'cs>) -> &'cs mut Self::Inner {
        let ptr = self.borrow(cs).get();
        let option_ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        option_ref
    }
}

pub trait IsSizeOne {}

impl IsSizeOne for u8 {}
impl IsSizeOne for i8 {}
impl IsSizeOne for bool {}

pub trait AtomicRead {
    type DataType;
    fn atomic_read(&self) -> Self::DataType;
}

impl<T> AtomicRead for Mutex<Cell<T>>
where
    // I would like to assert that T is size 1 at compile time, but I can't
    // so requiring the user to implement IsSizeOne to declare that the type
    // is the right size.
    T: Copy + IsSizeOne,
{
    type DataType = T;

    fn atomic_read(&self) -> Self::DataType {
        debug_assert!(core::mem::size_of::<T>() == 1);
        unsafe_access_mutex(|cs| self.borrow(cs).get())
    }
}
