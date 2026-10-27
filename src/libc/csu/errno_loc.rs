//! Safe wrappers over `csu/errno-loc.c`: the calling thread's `errno`, which C's
//! `errno` macro reaches through `__errno_location`.
//!
//! `errno` is thread-local, so reading and writing it races with nothing; every call made
//! between a read and a write may change it, as in C.

use core::ffi::c_int;

use crate::libc::ffi;

/// Wraps: `__errno_location`
///
/// The calling thread's `errno`: the error number the last failing call into C on this
/// thread left, as C's `errno` reads it.
#[must_use]
pub fn errno() -> c_int {
    // SAFETY: `__errno_location` returns the calling thread's `errno`, valid and aligned for
    // the thread's lifetime, which the read does not outlast.
    unsafe { location().read() }
}

/// Wraps: `__errno_location`
///
/// Sets the calling thread's `errno` to `value`, as C's `errno = value` does: before a call
/// whose failure only `errno` reports, or to restore the value a caller saw.
pub fn set_errno(value: c_int) {
    // SAFETY: as in `errno`: the calling thread's `errno`, valid for writes for the thread's
    // lifetime.
    unsafe { location().write(value) };
}

/// The calling thread's `errno`, valid for reads and writes for as long as the thread lives.
fn location() -> *mut c_int {
    // SAFETY: `__errno_location` takes no arguments and has no precondition.
    unsafe { ffi::__errno_location() }
}

#[cfg(test)]
mod unit_tests {
    use super::{errno, ffi, location, set_errno};

    #[test]
    fn set_errno_is_what_c_reads() {
        set_errno(ffi::ENOENT);
        // SAFETY: the calling thread's `errno`, valid for reads.
        assert_eq!(unsafe { location().read() }, ffi::ENOENT);
        assert_eq!(errno(), ffi::ENOENT);
        // SAFETY: as above, valid for writes.
        unsafe { location().write(ffi::EACCES) };
        assert_eq!(errno(), ffi::EACCES);
        set_errno(0);
        assert_eq!(errno(), 0);
    }
}
