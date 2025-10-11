// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use regex::Regex;
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

// Return PW_MAJOR, PW_MINOR and PW_MICRO  from pipewire/version.h
fn get_pw_version(pw_h_path: &PathBuf) -> (u32, u32, u32) {
    let pw_version_h_path = Path::new(pw_h_path).join("pipewire/version.h");
    assert!(
        pw_version_h_path.exists(),
        "{} doesn't exist",
        pw_version_h_path.display()
    );
    let header_content = read_to_string(pw_version_h_path).unwrap();
    let lines = header_content.lines();

    const VERSION_REGEX_STRINGS: [&str; 3] = [
        r"#define PW_MAJOR\s*[0-9]+",
        r"#define PW_MINOR\s*[0-9]+",
        r"#define PW_MICRO\s*[0-9]+",
    ];
    let mut numbers: [u32; 3] = [0; 3];
    for i in 0..3 {
        let re = Regex::new(VERSION_REGEX_STRINGS[i]).unwrap();
        let match_line = lines
            .clone()
            .filter(|&s| re.is_match(s))
            .collect::<Vec<_>>();
        assert_eq!(
            match_line.len(),
            1,
            "unexpected match for {}: {:?}",
            VERSION_REGEX_STRINGS[i],
            match_line
        );
        let number_str = Regex::new(r"[0-9]+")
            .unwrap()
            .find(match_line[0])
            .unwrap()
            .as_str();
        numbers[i] = number_str.parse::<u32>().unwrap();
    }

    (numbers[0], numbers[1], numbers[2])
}

fn main() {
    let libs = system_deps::Config::new()
        .probe()
        .expect("Cannot find libraries");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=discord_backfills.h");

    // Write bindings files to the $OUT_DIR/ directory.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let builder = bindgen::builder()
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Use `usize` for `size_t`. This behavior of bindgen changed because it is not
        // *technically* correct, but is the case in all architectures supported by Rust.
        .size_t_is_usize(true)
        .allowlist_function("spa_.*")
        .allowlist_type("spa_.*")
        .allowlist_var("SPA_.*")
        .prepend_enum_name(false)
        .derive_eq(true)
        // Create callable wrapper functions around SPAs `static inline` functions so they
        // can be called via FFI
        .wrap_static_fns(true)
        .wrap_static_fns_suffix("_libspa_rs")
        .wrap_static_fns_path(out_path.join("static_fns"));

    let builder = libs
        .iter()
        .iter()
        .flat_map(|(_, lib)| lib.include_paths.iter())
        .fold(builder, |builder, l| {
            let arg = format!("-I{}", l.to_string_lossy());
            builder.clang_arg(arg)
        });

    let builder = if cfg!(feature = "discord") {
        builder.clang_arg("-DDISCORD_BACKFILLS")
    } else {
        builder
    };

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    const FILES: &[&str] = &["src/type-info.c"];
    let cc_files = &[PathBuf::from(FILES[0]), out_path.join("static_fns.c")];

    for file in FILES {
        println!("cargo:rerun-if-changed={file}");
    }

    let pipewire = libs.get_by_name("libpipewire").unwrap();
    let header_dir = pipewire
        .include_paths
        .iter()
        .find(|&x| x.to_string_lossy().contains("pipe"))
        .unwrap();
    let (pw_major, pw_minor, pw_micro) = get_pw_version(header_dir);
    let pw_check_version = |desired_major: u32, desired_minor: u32, desired_micro: u32| {
        pw_major > desired_major
            || (pw_major == desired_major && pw_minor > desired_minor)
            || (pw_major == desired_major && pw_minor == desired_minor && pw_micro >= desired_micro)
    };
    println!("cargo::rustc-check-cfg=cfg(libpipewire_0_3_65_or_higher)");

    if pw_check_version(0, 3, 65) {
        println!("cargo::rustc-cfg=libpipewire_0_3_65_or_higher");
    }

    let mut cc = cc::Build::new();
    cc.files(cc_files);
    cc.include(env!("CARGO_MANIFEST_DIR"));
    cc.includes(libs.all_include_paths());

    cc.compile("libspa-rs-reexports");
}
