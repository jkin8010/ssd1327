// use std::{env, fs::File, io::Write, path::PathBuf};

// pub fn main() {
//     // Put the linker script somewhere the linker can find it
//     let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
//     File::create(out.join("memory.x"))
//         .unwrap()
//         .write_all(include_bytes!("memory.x"))
//         .unwrap();
//     println!("cargo:rustc-link-search={}", out.display());

//     println!("cargo:rerun-if-changed=build.rs");
//     println!("cargo:rerun-if-changed=memory.x");
// }

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> anyhow::Result<()> {
    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")
}
