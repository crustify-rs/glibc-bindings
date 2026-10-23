//! Safe Rust wrappers over glibc's C API, one module per link unit.

#[cfg(feature = "libc")]
pub mod libc;
