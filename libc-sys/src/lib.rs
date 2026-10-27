//! Raw bindgen bindings to glibc's `libc`.
//!
//! Generated from the host's headers through `bindgen/wrapper.h` under
//! `bindgen/allowlist.txt`, and shipped as `src/bindings.rs`, which the `bindgen` feature
//! regenerates against the host's glibc.
#![allow(
    nonstandard_style,
    missing_docs,
    dead_code,
    unused_imports,
    clippy::all,
    clippy::pedantic
)]

#[cfg(not(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")))]
compile_error!(
    "libc-sys supports x86_64-unknown-linux-gnu only, the target its bindings are generated for"
);

/// The version of glibc the bindings were generated from, as `getconf GNU_LIBC_VERSION`
/// reports it.
pub const C_VERSION: &str = env!("C_VERSION");

#[cfg(not(crustify_regenerated))]
include!("bindings.rs");
#[cfg(crustify_regenerated)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
