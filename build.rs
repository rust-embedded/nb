extern crate version_check;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    match version_check::Channel::read() {
        Some(c) if c.is_nightly() => println!("cargo:rustc-cfg=nightly"),
        _ => (),
    }
}
