fn main() {
    println!("cargo:rustc-link-search=native=./libs");
    println!("cargo:rustc-link-lib=dylib=cviwrapper");
    println!("cargo:rerun-if-changed=build.rs");
}
