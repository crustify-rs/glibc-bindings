//! Safe wrappers over `time/bits/types/struct_tm.h`, installed as
//! `<bits/types/struct_tm.h>`: [`Tm`] is C's `struct tm`, the broken-down time
//! `<time.h>` declares its conversions and formatting over. The header defines
//! no function, and no TU of glibc carries its name, so it is a header module of
//! its own.

use core::ffi::{CStr, c_int, c_long};
use core::fmt;
use core::ptr::{self, NonNull, addr_of, addr_of_mut};

use ffibox::{CMutSeam, CRefSeam, CZeroable, define_ctype};

use crate::libc::ffi;

define_ctype!(
    /// Wraps: `tm`
    ///
    /// C's `struct tm`, a broken-down time, which `strftime` formats and
    /// `mktime`, `gmtime_r` and their kin read and fill. A plain value Rust
    /// owns, built by [`Tm::new`] (or `Default`, or `zeroed()`) and filled
    /// through [`TmMut`]'s setters; its getters are on [`TmRef`].
    ///
    /// The fields are independent and unchecked, as in C: `strftime` reads
    /// those its conversions name and formats out-of-range values as the C
    /// library chooses. `tm_zone` is the one field with an invariant: NULL,
    /// or a `'static` string.
    #[derive(Clone, Copy)]
    Tm,
    TmRef,
    TmMut,
    ffi::tm
);

// SAFETY: all-zero is a NULL `tm_zone` and zero integers: every getter
// accepts it (`zone` reads NULL as `None`), and so does `strftime`, which
// substitutes the local zone's name (or nothing) for a NULL `tm_zone`; no
// teardown exists.
unsafe impl CZeroable for Tm {}

// SAFETY: the fields are integers and `tm_zone`, NULL or a `&'static CStr`
// (`Sync`, never written); nothing is freed or tied to a thread, so a value,
// or an exclusive handle to one, moves between threads as an
// `([i32; 9], i64, Option<&'static CStr>)` does.
unsafe impl Send for Tm {}
// SAFETY: as for `Tm`: the handle is the only access to the value.
unsafe impl Send for TmMut<'_> {}
// SAFETY: nothing reachable through a shared handle writes: `TmRef` has
// getters only, the struct has no interior mutability, and the zone string
// is never written.
unsafe impl Sync for Tm {}
// SAFETY: as for `Sync for Tm`: copies of a shared handle only read.
unsafe impl Send for TmRef<'_> {}
// SAFETY: as above.
unsafe impl Sync for TmRef<'_> {}
// SAFETY: `&TmMut` reaches the value only through `as_ref`, a shared handle;
// the setters need `&mut`.
unsafe impl Sync for TmMut<'_> {}

impl Tm {
    /// A zeroed time, as `zeroed()` builds it: every integer 0 (day 0 of
    /// January 1900, a Sunday, `tm_isdst` 0) and no zone name.
    #[must_use]
    pub fn new() -> Self {
        Self::zeroed()
    }
}

impl Default for Tm {
    /// [`Tm::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Tm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl fmt::Debug for TmRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tm")
            .field("sec", &self.sec())
            .field("min", &self.min())
            .field("hour", &self.hour())
            .field("mday", &self.mday())
            .field("mon", &self.mon())
            .field("year", &self.year())
            .field("wday", &self.wday())
            .field("yday", &self.yday())
            .field("isdst", &self.isdst())
            .field("gmtoff", &self.gmtoff())
            .field("zone", &self.zone())
            .finish()
    }
}

/// A getter on [`TmRef`] and a setter on [`TmMut`] for each integer field of
/// `struct tm`, read and written by value through the handle's pointer.
macro_rules! tm_fields {
    ($($(#[$doc:meta])* $field:ident: $ty:ty => $get:ident, $set:ident;)*) => {
        impl TmRef<'_> {
            $(
                $(#[$doc])*
                #[must_use]
                pub fn $get(&self) -> $ty {
                    // SAFETY: a read of an integer field projected through
                    // the shared handle's pointer to a live, initialised
                    // `struct tm`; every bit pattern is valid.
                    unsafe { addr_of!((*self.as_ptr()).$field).read() }
                }
            )*
        }

        impl TmMut<'_> {
            $(
                $(#[$doc])*
                pub fn $set(&mut self, value: $ty) {
                    // SAFETY: a write of an integer field projected through
                    // the exclusive handle's pointer to a live `struct tm`;
                    // the old value owns nothing to leak.
                    unsafe { addr_of_mut!((*self.as_mut_ptr()).$field).write(value) }
                }
            )*
        }
    };
}

tm_fields! {
    /// `tm_sec`: seconds after the minute, usually 0-60.
    tm_sec: c_int => sec, set_sec;
    /// `tm_min`: minutes after the hour, usually 0-59.
    tm_min: c_int => min, set_min;
    /// `tm_hour`: hours since midnight, usually 0-23.
    tm_hour: c_int => hour, set_hour;
    /// `tm_mday`: day of the month, usually 1-31.
    tm_mday: c_int => mday, set_mday;
    /// `tm_mon`: months since January, usually 0-11.
    tm_mon: c_int => mon, set_mon;
    /// `tm_year`: years since 1900.
    tm_year: c_int => year, set_year;
    /// `tm_wday`: days since Sunday, usually 0-6.
    tm_wday: c_int => wday, set_wday;
    /// `tm_yday`: days since January 1st, usually 0-365.
    tm_yday: c_int => yday, set_yday;
    /// `tm_isdst`: positive when daylight saving time is in effect, 0 when
    /// not, negative when unknown.
    tm_isdst: c_int => isdst, set_isdst;
    /// `tm_gmtoff`: seconds east of UTC, which `%z` formats.
    tm_gmtoff: c_long => gmtoff, set_gmtoff;
}

impl<'a> TmRef<'a> {
    /// `tm_zone`: the time zone's abbreviation, which `%Z` formats, or
    /// `None` when NULL. For `None` or an empty name, glibc's `strftime`
    /// names the local zone (`tzname[tm_isdst != 0]`) when `tm_isdst` is
    /// not negative, and nothing otherwise.
    ///
    /// ```
    /// use glibc_bindings::libc::time::bits::types::struct_tm_h::Tm;
    /// let mut tm = Tm::new();
    /// assert_eq!(tm.as_ref().zone(), None);
    /// tm.as_mut().set_zone(Some(c"XST"));
    /// assert_eq!(tm.as_ref().zone(), Some(c"XST"));
    /// ```
    #[must_use]
    pub fn zone(&self) -> Option<&'a CStr> {
        // SAFETY: the handle's pointer is to a live object, as the access below relies on,
        // so the place of its `tm_zone` field is in bounds; nothing is accessed here.
        let tm_zone_ptr = unsafe { addr_of!((*self.as_ptr()).tm_zone) };
        // SAFETY: a read of the `tm_zone` field projected through the shared
        // handle's pointer to a live, initialised `struct tm`.
        let zone = unsafe { tm_zone_ptr.read() };
        // SAFETY: every writer leaves `tm_zone` NULL (`zeroed`) or a
        // `&'static CStr` (`set_zone`), NUL-terminated and never written.
        NonNull::new(zone.cast_mut()).map(|p| unsafe { CStr::from_ptr(p.as_ptr()) })
    }
}

impl TmMut<'_> {
    /// `tm_zone`: points it at `zone`, or NULL for `None`. The value only
    /// borrows the string.
    pub fn set_zone(&mut self, zone: Option<&'static CStr>) {
        let zone = zone.map_or(ptr::null(), CStr::as_ptr);
        // SAFETY: the handle's pointer is to a live object, as the access below relies on,
        // so the place of its `tm_zone` field is in bounds; nothing is accessed here.
        let tm_zone_ptr = unsafe { addr_of_mut!((*self.as_mut_ptr()).tm_zone) };
        // SAFETY: a write of the `tm_zone` field projected through the
        // exclusive handle's pointer to a live `struct tm`; the old value is
        // a borrow, so nothing leaks, and the new one lives for the program.
        unsafe { tm_zone_ptr.write(zone) }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn tm_new_default_and_zeroed_are_empty() {
        for t in [Tm::new(), Tm::default(), Tm::zeroed()] {
            let r = t.as_ref();
            let ints = [r.sec(), r.min(), r.hour(), r.mday(), r.mon()];
            assert_eq!(ints, [0; 5]);
            assert_eq!([r.year(), r.wday(), r.yday(), r.isdst()], [0; 4]);
            assert_eq!((r.gmtoff(), r.zone()), (0, None));
        }
    }

    #[test]
    fn tm_setters_write_their_own_field_only() {
        let mut t = Tm::new();
        let mut m = t.as_mut();
        m.set_sec(1);
        m.set_min(2);
        m.set_hour(3);
        m.set_mday(4);
        m.set_mon(5);
        m.set_year(c_int::MIN);
        m.set_wday(7);
        m.set_yday(c_int::MAX);
        m.set_isdst(-1);
        m.set_gmtoff(c_long::MIN);
        m.set_zone(Some(c"CET"));
        let raw = t.0;
        assert_eq!(
            [raw.tm_sec, raw.tm_min, raw.tm_hour, raw.tm_mday, raw.tm_mon],
            [1, 2, 3, 4, 5]
        );
        assert_eq!(
            [raw.tm_year, raw.tm_wday, raw.tm_yday, raw.tm_isdst],
            [c_int::MIN, 7, c_int::MAX, -1]
        );
        assert_eq!(raw.tm_gmtoff, c_long::MIN);
        assert_eq!(raw.tm_zone, c"CET".as_ptr());
        let r = t.as_ref();
        assert_eq!(
            (r.year(), r.yday(), r.gmtoff()),
            (c_int::MIN, c_int::MAX, c_long::MIN)
        );
        assert_eq!(r.zone(), Some(c"CET"));
        t.as_mut().set_zone(None);
        assert!(t.0.tm_zone.is_null());
        assert_eq!(t.as_ref().zone(), None);
    }

    #[test]
    fn tm_debug_lists_every_field() {
        let mut t = Tm::new();
        t.as_mut().set_year(100);
        t.as_mut().set_zone(Some(c"UTC"));
        assert_eq!(
            format!("{t:?}"),
            "Tm { sec: 0, min: 0, hour: 0, mday: 0, mon: 0, year: 100, wday: 0, yday: 0, \
             isdst: 0, gmtoff: 0, zone: Some(\"UTC\") }"
        );
        assert_eq!(format!("{:?}", t.as_ref()), format!("{t:?}"));
    }
}
