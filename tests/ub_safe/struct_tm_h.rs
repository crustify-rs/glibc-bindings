//! UB tests for `time/bits/types/struct_tm.h`'s [`Tm`], through the safe API
//! only: its asserted `Send` and `Sync`, with values, copies and handles used
//! from other threads.
//!
//! [`Tm`]: glibc_bindings::libc::time::bits::types::struct_tm_h::Tm

use std::thread;

use glibc_bindings::libc::time::bits::types::struct_tm_h::Tm;

/// A time with every field set, the zone among them.
fn some_time(k: i32) -> Tm {
    let mut tm = Tm::new();
    let mut m = tm.as_mut();
    m.set_sec(k);
    m.set_year(124);
    m.set_isdst(-1);
    m.set_gmtoff(3600);
    m.set_zone(Some(c"CET"));
    tm
}

#[test]
fn values_and_handles_cross_threads() {
    let mut tm = some_time(1);
    let copies: Vec<Tm> = (0..4).map(|_| tm).collect();
    let seen: Vec<_> = copies
        .into_iter()
        .map(|t| thread::spawn(move || (t.as_ref().sec(), t.as_ref().zone() == Some(c"CET"))))
        .map(|h| h.join().unwrap())
        .collect();
    assert!(seen.iter().all(|&s| s == (1, true)));
    let shared = tm.as_ref();
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(move || assert_eq!(shared.gmtoff(), 3600));
        }
    });
    let mut exclusive = tm.as_mut();
    thread::scope(|s| {
        s.spawn(|| exclusive.set_sec(2));
    });
    assert_eq!(tm.as_ref().sec(), 2);
}

#[test]
fn threads_build_and_fill_their_own_times() {
    let times: Vec<Tm> = (0..8)
        .map(|k| thread::spawn(move || some_time(k)))
        .map(|h| h.join().unwrap())
        .collect();
    for (k, t) in (0..).zip(&times) {
        assert_eq!((t.as_ref().sec(), t.as_ref().year()), (k, 124));
    }
}
