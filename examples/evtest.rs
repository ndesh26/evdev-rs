extern crate evdev;

use evdev::*;
use std::fs::File;

fn usage() {
    println!("Usage: evtest /path/to/device");
}

fn main() {
    let mut args = std::env::args();

    if args.len() != 2 {
        usage();
        std::process::exit(1);
    }

    let path = &args.nth(1).unwrap();
    let f = File::open(path).unwrap();

    let mut d = Device::new();
    d.set_fd(&f).unwrap();

    println!("Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
			d.bustype(),
			d.vendor_id(),
			d.product_id());
    println!("Evdev version: {:x}", d.driver_version());
    println!("Input device name: \"{}\"", d.name().unwrap_or(""));
    println!("Phys location: {}", d.phys().unwrap_or(""));
    println!("Uniq identifier: {}", d.uniq().unwrap_or(""));
}
