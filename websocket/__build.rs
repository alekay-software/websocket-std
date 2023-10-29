// extern crate cbindgen;

// use std::env;

// fn main() {
//     let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

//     cbindgen::Builder::new()
//       .with_crate(crate_dir)
//       .generate()
//       .expect("Unable to generate bindings")
//       .write_to_file("bindings.h");
// }

// extern crate cbindgen;

// use std::env;

// fn main() {
//     let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
//     let mut config: cbindgen::Config = Default::default();
//     config.language = cbindgen::Language::Cxx;
//     cbindgen::generate_with_config(&crate_dir, config)
//       .unwrap()
//       .write_to_file("target/websocket_std.h");
// }

// extern crate cbindgen;
// extern crate cc;

// use std::env;
// use std::path::PathBuf;

// fn main() {
//     let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
//     let cbindgen_toml = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("cbindgen.toml");

//     let bindings = cbindgen::Builder::new()
//         .with_config(cbindgen::Config::from_file(&cbindgen_toml).unwrap())
//         // This wouldn't be needed if it was a library crate or something.
//         .generate()
//         .unwrap();

//     let _changed = bindings.write_to_file(out_dir.join("my_ffi.h"));

//     let cpp_file = "cpp/doit.cpp";

//     cc::Build::new()
//         .cpp(true)
//         .file(cpp_file)
//         .flag("-I")
//         .flag(&format!("{}", out_dir.display()))
//         .flag("-I")
//         .flag("cpp")
//         .flag("-std=c++20")
//         .compile("libdoit.a");

//     println!("cargo:rerun-if-changed={}", cbindgen_toml.display());
//     println!("cargo:rerun-if-changed={}", cpp_file);
// }

// fn main() {
//     let _build = cxx_build::bridge("src/lib.rs");
//     println!("cargo:rerun-if-changed=src/lib.rs");
// }