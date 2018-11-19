//! # ipu-demo build script for P3411 Wireless Embedded / P3642 Cerberust
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! This crate is a build script which generates version information and
//! places it in `src/version/data.rs`.

use std::process::Command;

fn main() {
    // Put the linker script somewhere the linker can find it
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=am5728_ipu.ld");
    let git_desc = Command::new("git")
        .args(&["describe", "--all", "--dirty", "--tags", "--long"])
        .output()
        .unwrap();
    let git_describe = String::from_utf8_lossy(&git_desc.stdout);
    let package_name = env!("CARGO_PKG_NAME");
    let version = format!("{0} ({1})\n", package_name.trim(), git_describe.trim());
    let code = format!(
        "
#[link_section = \".version\"]
pub static VERSION_TEXT: [u8; {0}] = *b{1:?};
",
        version.len(),
        version
    );
    std::fs::write("src/version/data.rs", code).unwrap();
}
