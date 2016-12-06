fn main() {
    println!("cargo:rustc-link-search=usr/lib64/");
    println!("cargo:rustc-link-lib=evdev");
}
