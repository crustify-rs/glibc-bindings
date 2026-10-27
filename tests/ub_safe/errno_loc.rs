//! `errno` through the safe API: per thread, and set by C's own failures.

use std::thread;

use glibc_bindings::libc::csu::errno_loc::{errno, set_errno};
use glibc_bindings::libc::ffi::{EACCES, ENOENT};

#[test]
fn errno_round_trips() {
    set_errno(ENOENT);
    assert_eq!(errno(), ENOENT);
    set_errno(0);
    assert_eq!(errno(), 0);
}

#[test]
fn each_thread_has_its_own_errno() {
    set_errno(ENOENT);
    let threads: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                set_errno(EACCES);
                for _ in 0..1000 {
                    assert_eq!(errno(), EACCES);
                }
                errno()
            })
        })
        .collect();
    for t in threads {
        assert_eq!(t.join().unwrap(), EACCES);
    }
    assert_eq!(errno(), ENOENT);
}

#[test]
#[cfg_attr(miri, ignore = "opens a file, which Miri's isolation refuses")]
fn a_failing_c_call_sets_errno() {
    set_errno(0);
    // A missing file: std's `open` reports libc's errno, which it leaves set on the thread.
    let err = std::fs::File::open("/nonexistent/crustify").unwrap_err();
    assert_eq!(err.raw_os_error(), Some(ENOENT));
    assert_eq!(errno(), ENOENT);
}
