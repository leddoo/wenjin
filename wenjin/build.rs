use std::env;

pub fn main() {
    // an alternative to `cfg(debug_assertions)`.
    if env::var("PROFILE").as_deref() == Ok("release") {
        println!("cargo:rustc-cfg=wenjin_paranoia=\"nay\"")
    }
    else {
        // comment out this line to run the fast/unchecked code in miri.
        println!("cargo:rustc-cfg=wenjin_paranoia=\"yas\"")
    }
}

