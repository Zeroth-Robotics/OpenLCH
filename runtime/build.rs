fn main() {
    #[cfg(feature = "milkv")]
    {
        println!("cargo:rustc-link-search=native=./firmware/cviwrapper");
        println!("cargo:rustc-link-lib=dylib=cviwrapper");
        tonic_build::compile_protos("proto/hal.proto")?;
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}
