#![feature(path_file_prefix)]

use libbpf_cargo::SkeletonBuilder;
use std::env;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));

    println!("cargo:rerun-if-changed=src/bpf/*");
    for path in std::fs::read_dir(PathBuf::from("src/bpf/")).unwrap() {
        println!("in bpf file: {:?}", path);
        let Ok(path) = path else {
            continue;
        };

        let file_name = path.file_name().to_string_lossy().to_string();
        let path = path.path();
        if path.is_dir() {
            continue;
        }

        let Some((file_name, extend)) = file_name.split_once('.') else {
            continue;
        };
        println!("in bpf file: {:?}", path);
        if !extend.eq("bpf.c") {
            continue;
        }
        let mut out = out.clone();
        out.push(format!("{}.skel.rs", file_name));

        println!("out bpf file: {:?}", out);

        SkeletonBuilder::new()
            .source(path)
            .clang_args(["-Wall", "-Wno-compare-distinct-pointer-types"])
            .build_and_generate(&out)
            .unwrap();
    }
}
