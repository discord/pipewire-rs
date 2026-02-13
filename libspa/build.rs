use regex::Regex;
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
    // FIXME: It would be nice to run this only when tests are run.
    println!("cargo:rerun-if-changed=tests/pod.c");

    let libs = system_deps::Config::new()
        .probe()
        .expect("Cannot find libspa");
    let libspa = libs.get_by_name("libspa").unwrap();

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

    cc::Build::new()
        .file("tests/pod.c")
        .shared_flag(true)
        .flag("-Wno-missing-field-initializers")
        .includes(&libspa.include_paths)
        .compile("pod");
}
