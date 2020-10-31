extern crate bindgen;

use std::env;
// use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=SimpleMotionV2/simplemotion.h");
    println!("cargo:rerun-if-changed=SimpleMotionV2/simplemotion.c");

    // fs::copy("patch/config.h", "linuxcnc-src/src/config.h")
    //     .expect("Failed to copy config patch file");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .files(&[
            "SimpleMotionV2/simplemotion.c",
            "SimpleMotionV2/sm_consts.c",
            "SimpleMotionV2/busdevice.c",
            "SimpleMotionV2/drivers/serial/pcserialport.c",
            "SimpleMotionV2/drivers/tcpip/tcpclient.c",
            "SimpleMotionV2/utils/crc.c",
        ])
        .include("SimpleMotionV2")
        .include("SimpleMotionV2/utils")
        .include("SimpleMotionV2/drivers")
        .warnings(false)
        .define("ENABLE_BUILT_IN_DRIVERS", Some("1"))
        .define("ENABLE_DEBUG_PRINTS", None)
        .compile("simplemotionv2");
}
