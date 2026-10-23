//! The `libc` link unit, laid out as glibc's C tree: one module per C directory.

/// Raw bindings to `libc`, generated from the host's headers.
pub use libc_sys as ffi;

pub mod time;
