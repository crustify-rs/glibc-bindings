//! Generates the raw bindings from the host's glibc headers, restricted to
//! `allowlist.txt`. The library is the system's, which every Rust binary links already,
//! so nothing is built or linked here. Layouts that differ between 64-bit architectures
//! come out of the host's headers, for the architecture the crate builds on.

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let allowlist = manifest.join("allowlist.txt");
    let wrapper = manifest.join("wrapper.h");
    println!("cargo:rerun-if-changed={}", allowlist.display());
    println!("cargo:rerun-if-changed={}", wrapper.display());
    let mut b = bindgen::Builder::default()
        .header(wrapper.to_str().unwrap())
        .derive_default(true)
        .generate_comments(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    for line in std::fs::read_to_string(&allowlist).unwrap().lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let (kind, pat) = line
            .split_once(char::is_whitespace)
            .expect("`<kind> <regex>`");
        let pat = pat.trim();
        b = match kind {
            "function" => b.allowlist_function(pat),
            "type" => b.allowlist_type(pat),
            "var" => b.allowlist_var(pat),
            "item" => b.allowlist_item(pat),
            k => panic!("allowlist.txt: unknown kind {k:?}"),
        };
    }
    b.generate()
        .expect("bindgen")
        .write_to_file(out.join("bindings.rs"))
        .expect("write bindings.rs");
}
