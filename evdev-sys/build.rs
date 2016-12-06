fn main() {
    // TODO: build libevdev ourselves in case it is not present
    println!("cargo:rustc-link-search=usr/lib64/");
    println!("cargo:rustc-link-lib=evdev");
}
