// ==========================================
// UloOS Kernel Build Script
// ==========================================
// Compiles the doomgeneric C sources into a static library that gets
// linked into the Rust kernel binary.
//
// Before building, run setup_doom.ps1 to download the doomgeneric source
// and DOOM1.WAD shareware file.

use std::path::Path;
use std::process::Command;

fn main() {
    // Tell cargo about our custom cfg flag
    println!("cargo::rustc-check-cfg=cfg(no_doom_engine)");
    println!("cargo:rustc-cfg=no_doom_engine");
}
