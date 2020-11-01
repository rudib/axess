extern crate vergen;

use std::env;
use vergen::{ConstantsFlags, generate_cargo_keys};

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=build_profile={:?}", profile);
        println!("cargo:rustc-env=BUILD_PROFILE={}", profile);
    }
    
    // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
    let mut flags = ConstantsFlags::all();
    flags.toggle(ConstantsFlags::SEMVER_FROM_CARGO_PKG);

    // Generate the 'cargo:' key output
    generate_cargo_keys(ConstantsFlags::all()).expect("Unable to generate the cargo keys!");
}