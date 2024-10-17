fn main() {
    #[cfg(feature = "milkv")]
    {
        println!("cargo:rustc-link-search=native=./firmware/cviwrapper");
        println!("cargo:rustc-link-lib=dylib=cviwrapper");
    }

    use tonic_build;
    tonic_build::compile_protos("proto/hal_pb.proto").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/*");
}
