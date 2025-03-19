use std::env;

use rustc_version::version_meta;
use rustc_version::Channel::Nightly;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nightly_rustc)");
    if version_meta().unwrap().channel <= Nightly {
        println!("cargo:rustc-cfg=nightly_rustc");
    }

    println!("cargo:rustc-check-cfg=cfg(ci)");
    if env::var("CI").is_ok() {
        println!("cargo:rustc-cfg=ci")
    }
}
