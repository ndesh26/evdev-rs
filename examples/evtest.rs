extern crate evdev;
extern crate nix;

use evdev::*;
use nix::errno::Errno;
use std::fs::File;

fn usage() {
    println!("Usage: evtest /path/to/device");
}
/*
fn print_abs_bits(dev: &Device, axis: u32) {

	if !dev.has_event_code(consts::EV::EV_ABS, axis) { return; }

	let abs = dev.abs_info(axis).unwrap();

	println!("	Value	{}", abs.value);
	println!("	Min	{}", abs.minimum);
	println!("	Max	{}", abs.maximum);
	if abs.fuzz != 0 {
		println!("	Fuzz	{}", abs.fuzz);
    }
	if abs.flat != 0 {
		println!("	Flat	{}", abs.flat);
    }
	if abs.resolution != 0 {
		println!("	Resolution	{}", abs.resolution);
    }
}

fn print_code_bits(dev: &Device, event_type: u32, max: u32) {
    for i in  0..max {
		if !dev.has_event_code(event_type, i) { continue; }

		println!("    Event code {} ({})", i, event_code_get_name(event_type, i).unwrap());
		if event_type == consts::EV::EV_ABS as u32 {
			print_abs_bits(dev, i);
	    }
    }
}

fn print_bits(dev: &Device) {
    println!("Supported events:");

    for i in  0..consts::EV::EV_MAX as u32 {
		if dev.has_event_type(i) {
			println!("  Event type {} ({})", i, event_type_get_name(i).unwrap());
        }

        let event_type: consts::EV = unsafe { std::mem::transmute(i as u8) };
		match event_type {
		    consts::EventType::EV_KEY => print_code_bits(dev, consts::EV::EV_KEY as u32,
                                                  consts::KEY::KEY_MAX as u32),
			consts::EventType::EV_REL => print_code_bits(dev, consts::EV::EV_REL as u32,
                                                  consts::REL::REL_MAX as u32),
			consts::EventType::EV_ABS => print_code_bits(dev, consts::EV::EV_ABS as u32,
                                                  consts::ABS::ABS_MAX as u32),
			consts::EventType::EV_LED => print_code_bits(dev, consts::EV::EV_LED as u32,
                                                  consts::LED::LED_MAX as u32),
            _ => (),
		}
	}
}
*/
fn print_props(dev: &Device) {
	println!("Properties:");

	for i in 0..consts::INPUT_PROP::INPUT_PROP_MAX as u32 {
		if dev.has_property(i) {
			println!("  Property type {} ({})", i, property_get_name(i).unwrap());
        }
    }
}

fn print_event(ev: &InputEvent) {
    match ev.event_type {
        consts::EventType::EV_SYN => {
		    println!("Event: time {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
				     ev.time.tv_sec,
				     ev.time.tv_usec,
				     ev.event_type);
        }
	    _ =>  {
		    println!("Event: time {}.{}, type {} , code {} , value {}",
			         ev.time.tv_sec,
			         ev.time.tv_usec,
			         ev.event_type,
			         ev.event_code,
			         ev.value);
        }
    }
}

fn print_sync_event(ev: &InputEvent) {
	print!("SYNC: ");
	print_event(ev);
}

fn main() {
    let mut args = std::env::args();

    if args.len() != 2 {
        usage();
        std::process::exit(1);
    }

    let path = &args.nth(1).unwrap();
    let f = File::open(path).unwrap();

    let mut d = Device::new().unwrap();
    d.set_fd(&f).unwrap();

    println!("Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
			d.bustype(),
			d.vendor_id(),
			d.product_id());
    println!("Evdev version: {:x}", d.driver_version());
    println!("Input device name: \"{}\"", d.name().unwrap_or(""));
    println!("Phys location: {}", d.phys().unwrap_or(""));
    println!("Uniq identifier: {}", d.uniq().unwrap_or(""));

	//print_bits(&d);
    //print_props(&d);

    let mut a: Result<(ReadStatus, InputEvent), Errno>;
    loop {
        a = d.next_event(evdev::NORMAL | evdev::BLOCKING);
        if a.is_ok() {
            let mut result = a.ok().unwrap();
            match result.0 {
                ReadStatus::Sync => {
                    println!("::::::::::::::::::::: dropped ::::::::::::::::::::::");
                    while result.0 == ReadStatus::Sync {
                        print_sync_event(&result.1);
                        a = d.next_event(evdev::SYNC);
                        if a.is_ok() {
                            result = a.ok().unwrap();
                        } else {
                            break;
                        }
                    }
                    println!("::::::::::::::::::::: re-synced ::::::::::::::::::::");
                },
                ReadStatus::Success => print_event(&result.1),
            }
        } else {
            match a.err().unwrap() {
                Errno::EAGAIN => continue,
                err => println!("{}", err),
            }
        }
    }
}
