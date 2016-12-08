extern crate pkg_config;

fn main() {

   if let Ok(lib) = pkg_config::find_library("libevdev") {
        for path in &lib.include_paths {
            println!("cargo:include={}", path.display());
        }
        return
    }

    // TODO: build libevdev ourselves in case it is not present
}
