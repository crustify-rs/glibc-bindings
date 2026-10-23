//! Raw bindgen bindings to glibc.
//!
//! Generated from the host's headers through `wrapper.h`, under `allowlist.txt`.
#![allow(
    nonstandard_style,
    missing_docs,
    dead_code,
    unused_imports,
    clippy::all,
    clippy::pedantic
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
