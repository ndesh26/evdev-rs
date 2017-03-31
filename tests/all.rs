extern crate evdev;

use evdev::*;
use std::fs::File;
use std::os::unix::io::AsRawFd;

#[test]
#[allow(dead_code)]
fn context_create() {
    Device::new().unwrap();
}

#[test]
fn context_create_with_fd() {
    let f = File::open("/dev/input/event0").unwrap();
    let mut d = Device::new_from_fd(&f).unwrap();
}

#[test]
fn context_set_fd() {
    let mut d = Device::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    match d.set_fd(&f) {
        Ok(()) => ..,
        Err(result) => panic!("Error {}", result.desc()),
    };
}

#[test]
fn context_change_fd() {
    let mut d = Device::new().unwrap();
    let f1 = File::open("/dev/input/event0").unwrap();
    let f2 = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f1).unwrap();
    match d.change_fd(&f2) {
        Ok(()) => ..,
        Err(result) => panic!("Error {}", result),
    };

    assert_eq!(d.fd().unwrap().as_raw_fd(), f2.as_raw_fd());
}

#[test]
fn context_grab() {
    let mut d = Device::new().unwrap();
    let mut f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    d.grab(GrabMode::Grab).unwrap();
    d.grab(GrabMode::Ungrab).unwrap();
}

#[test]
fn device_get_name() {
    let d = Device::new().unwrap();

    d.set_name("hello");
    assert_eq!(d.name().unwrap(), "hello");
}

#[test]
fn device_get_uniq() {
    let d = Device::new().unwrap();

    d.set_uniq("test");
    assert_eq!(d.uniq().unwrap(), "test");
}

#[test]
fn device_get_phys() {
    let d = Device::new().unwrap();

    d.set_phys("test");
    assert_eq!(d.phys().unwrap(), "test");
}

#[test]
fn device_get_product_id() {
    let d = Device::new().unwrap();

    d.set_product_id(5);
    assert_eq!(d.product_id(), 5);
}

#[test]
fn device_get_vendor_id() {
    let d = Device::new().unwrap();

    d.set_vendor_id(5);
    assert_eq!(d.vendor_id(), 5);
}

#[test]
fn device_get_bustype() {
    let d = Device::new().unwrap();

    d.set_bustype(5);
    assert_eq!(d.bustype(), 5);
}

#[test]
fn device_get_version() {
    let d = Device::new().unwrap();

    d.set_version(5);
    assert_eq!(d.version(), 5);
}

#[test]
fn device_get_absinfo() {
    let mut d = Device::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    for code in 0..0xff {
        let absinfo: Option<AbsInfo> = d.abs_info(code);

        match absinfo {
            None => ..,
            Some(a) => ..,
        };
    }
}

#[test]
fn device_has_property() {
    let mut d = Device::new().unwrap();
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
    let mut d = Device::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    /*for t in 0x18..0xff {
        if d.has_event_type(t) {
            panic!("Type {} is set, shouldn't be", t);
        }
        for c in 0x00..0xff {
            if d.has_event_code(t, c) {
                panic!("Type {} Code {} is set, shouldn't be", t, c);
            }
        }
    }*/
}

#[test]
fn device_has_syn() {
    let mut d = Device::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();

    assert!(d.has_event_type(consts::EventType::EV_SYN)); // EV_SYN
    assert!(d.has_event_code(consts::EventCode::EV_SYN(consts::SYN::SYN_REPORT))); // SYN_REPORT
}

#[test]
fn device_get_value() {
    let mut d = Device::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();

    let v2 = d.event_value(consts::EventCode::EV_SYN(consts::SYN::SYN_REPORT)); // SYN_REPORT
    assert_eq!(v2, Some(0));
}

#[test]
fn check_event_name() {
   assert_eq!("EV_ABS", event_type_get_name(consts::EventType::EV_ABS).unwrap());
}
