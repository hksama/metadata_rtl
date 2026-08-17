use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.parent().expect("rust crate lives under kv-rtl/");
    let build_dir = root.join("build");
    let verilator_dir = build_dir.join("verilator");
    let _verilator_include = PathBuf::from(
        env::var("VERILATOR_INCLUDE").unwrap_or_else(|_| "/usr/share/verilator/include".into()),
    );

    if !verilator_dir.join("Vtop__ALL.a").exists() {
        let status = Command::new("make")
            .arg("verilate")
            .current_dir(&root)
            .status()
            .expect("failed to invoke make verilate");
        if !status.success() {
            panic!("make verilate failed");
        }
    }

    println!("cargo:rerun-if-changed={}", root.join("rtl/kv_pkg.sv").display());
    println!("cargo:rerun-if-changed={}", root.join("rtl/kv_page_tracker.sv").display());
    println!("cargo:rerun-if-changed={}", root.join("sim/simulator.cpp").display());
    println!("cargo:rerun-if-changed={}", root.join("sim/simulator.h").display());

    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-search=native={}", verilator_dir.display());
    println!("cargo:rustc-link-lib=static=kvsim");
    println!("cargo:rustc-link-lib=static=Vtop");
    println!("cargo:rustc-link-lib=static=verilated");

    println!("cargo:rustc-link-lib=stdc++");
}
