extern crate pkg_config;
extern crate gcc;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(e) => panic!("{} return the error {}", stringify!($e), e),
    })
}

fn main() {

    if let Ok(lib) = pkg_config::find_library("libevdev") {
        for path in &lib.include_paths {
            println!("cargo:include={}", path.display());
        }
        return
    }

    if !Path::new("libevdev/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
                                   .status();
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let src = t!(env::current_dir());

    println!("cargo:rustc-link-search={}/lib", dst.display());
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
    println!("cargo:rerun-if-changed=libevdev/autogen.sh");

    println!("cargo:rustc-link-lib=static=evdev");
    let cfg = gcc::Config::new();
    let compiler = cfg.get_compiler();

    let _ = fs::create_dir(&dst.join("build"));

    let mut cmd = Command::new("sh");
    let mut cflags = OsString::new();
    for arg in compiler.args() {
        cflags.push(arg);
        cflags.push(" ");
    }
    cmd.env("CC", compiler.path())
       .env("CFLAGS", cflags)
       .current_dir(&dst.join("build"))
       .arg(src.join("libevdev/autogen.sh").to_str().unwrap()
               .replace("C:\\", "/c/")
               .replace("\\", "/"));
    cmd.arg(format!("--prefix={}", sanitize_sh(&dst)));

    run(&mut cmd);
    run(Command::new("make")
                .arg(&format!("-j{}", env::var("NUM_JOBS").unwrap()))
                .current_dir(&dst.join("build")));
    run(Command::new("make")
                .arg("install")
                .current_dir(&dst.join("build")));
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    assert!(t!(cmd.status()).success());
}

fn sanitize_sh(path: &Path) -> String {
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None
        }
        if ch.next() != Some('/') {
            return None
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}
