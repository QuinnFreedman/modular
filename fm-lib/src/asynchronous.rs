use core::{
    arch::asm,
    sync::atomic::{compiler_fence, Ordering},
};

use avr_device::interrupt::CriticalSection;

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
