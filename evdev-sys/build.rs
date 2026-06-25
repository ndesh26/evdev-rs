use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    if env::var("SYSTEM_DEPS_LIBEVDEV_BUILD_INTERNAL").is_err() {
        unsafe { env::set_var("SYSTEM_DEPS_LIBEVDEV_BUILD_INTERNAL", "auto") };
    }
    system_deps::Config::new()
        .add_build_internal("libevdev", build_from_source)
        .probe()
        .unwrap();
}

fn build_from_source(
    lib: &str,
    version: &str,
) -> Result<system_deps::Library, system_deps::BuildInternalClosureError> {
    if !std::path::Path::new("libevdev/.git").exists() {
        let _ = Command::new("git")
            .args(["submodule", "update", "--init", "--depth", "1"])
            .status();
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build_dir = dst.join("build");
    let src = env::current_dir().unwrap();
    let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".into());

    let mut cp = Command::new("cp");
    run(cp.arg("-r").arg(src.join("libevdev/")).arg(&dst))?;

    if !build_dir.exists() {
        fs::create_dir(&build_dir)
            .map_err(|e| system_deps::BuildInternalClosureError::failed(&e.to_string()))?;
    }

    let mut autogen = Command::new("sh");
    autogen
        .current_dir(&build_dir)
        .arg(dst.join("libevdev/autogen.sh").to_str().unwrap())
        .arg(format!("--prefix={}", dst.display()));
    if let Ok(h) = env::var("HOST") {
        autogen.arg(format!("--host={}", h));
    }
    if let Ok(t) = env::var("TARGET") {
        autogen.arg(format!("--target={}", t));
    }
    run(&mut autogen)?;

    let mut make = Command::new("make");
    run(make.arg(format!("-j{jobs}")).current_dir(&build_dir))?;
    let mut install = Command::new("make");
    run(install.arg("install").current_dir(&build_dir))?;

    println!("cargo:rerun-if-changed=libevdev");

    system_deps::Library::from_internal_pkg_config(dst.join("lib/pkgconfig"), lib, version)
}

fn run(cmd: &mut Command) -> Result<(), system_deps::BuildInternalClosureError> {
    cmd.status()
        .ok()
        .and_then(|s| s.success().then_some(()))
        .ok_or_else(|| system_deps::BuildInternalClosureError::failed(&format!("{cmd:?} failed")))
}
