//! Build script for leansig-script.
//!
//! This builds the guest program and sets the environment variable needed by include_elf!

fn main() {
    // Build the guest program
    sp1_build::build_program("../program");
}
