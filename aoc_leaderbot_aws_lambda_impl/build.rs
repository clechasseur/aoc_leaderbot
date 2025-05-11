use std::env;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(ci)");
    if env::var("CI").is_ok() {
        println!("cargo:rustc-cfg=ci")
    }
}
