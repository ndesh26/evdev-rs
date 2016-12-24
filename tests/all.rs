extern crate evdev;

use evdev::*;
use std::fs::File;

#[test]
#[allow(dead_code)]
fn context_create() {
    Device::new();
}

#[test]
fn context_set_fd() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    match d.set_fd(&f) {
        Ok(()) => ..,
        Err(result) => panic!("Error {}", result.desc()),
    };
}

#[test]
fn context_change_fd() {
    let mut d = Device::new();
    let f1 = File::open("/dev/input/event0").unwrap();
    let f2 = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f1).unwrap();
    match d.change_fd(&f2) {
        Ok(()) => ..,
        Err(result) => panic!("Error {}", result.desc()),
    };
}

#[test]
fn context_grab() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    d.grab(GrabMode::Grab).unwrap();
    d.grab(GrabMode::Ungrab).unwrap();
}

#[test]
fn device_get_name() {
    let mut d = Device::new();

    d.set_name("hello");
    match d.name() {
        Some("hello") => (),
        _ => panic!("Invalid name"),
    }
}

#[test]
fn device_get_uniq() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    match d.uniq() {
        _ => ..,
    };
}

#[test]
fn device_get_phys() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    match d.phys() {
        _ => ..,
    };
}

#[test]
fn device_get_absinfo() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    for code in 0..0xff {
        let absinfo: Option<AbsInfo> = d.get_abs_info(code);

        match absinfo {
            None => ..,
            Some(a) => ..,
        };
    }
}

#[test]
fn device_has_property() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    for prop in 0..0xff {
        if d.has_property(prop) && prop > 4 {
            panic!("Prop {} is set, shouldn't be", prop);
        }
    }
}

#[test]
fn device_has_type_code() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    for t in 0x18..0xff {
        if d.has_event_type(t) {
            panic!("Type {} is set, shouldn't be", t);
        }
        for c in 0x00..0xff {
            if d.has_event_code(t, c) {
                panic!("Type {} Code {} is set, shouldn't be", t, c);
            }
        }
    }
}

#[test]
fn device_has_syn() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();

    assert!(d.has_event_type(0)); // EV_SYN
    assert!(d.has_event_code(0, 0)); // SYN_REPORT
}

#[test]
fn device_get_value() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();

    let v1 = d.get_event_value(0xff, 0xff); // garbage
    assert_eq!(v1, None);
    let v2 = d.get_event_value(consts::EV::EV_SYN as u32, consts::SYN::SYN_REPORT as u32); // SYN_REPORT
    assert_eq!(v2, Some(0));
}
