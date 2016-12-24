extern crate evdev_sys as raw;
extern crate nix;
extern crate libc;

pub mod consts;
pub mod log;

use libc::{c_char};
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::fs::File;
use std::ffi::CStr;
use consts::*;

#[derive(Copy)]
#[derive(Clone)]
pub enum BusType {
    USB,
}

pub enum GrabMode {
    Grab = raw::LIBEVDEV_GRAB as isize,
    Ungrab = raw::LIBEVDEV_UNGRAB as isize,
}

pub enum ReadFlag {
    Sync = raw::LIBEVDEV_READ_FLAG_SYNC as isize,
    Normal = raw::LIBEVDEV_READ_FLAG_NORMAL as isize,
    ForceSync = raw::LIBEVDEV_READ_FLAG_FORCE_SYNC as isize,
    Blocking = raw::LIBEVDEV_READ_FLAG_BLOCKING as isize,
}

pub enum ReadStatus {
    Success = raw::LIBEVDEV_READ_STATUS_SUCCESS as isize,
    Sync = raw::LIBEVDEV_READ_STATUS_SYNC as isize,
}

pub enum LedState {
    On = raw::LIBEVDEV_LED_ON as isize,
    Off = raw::LIBEVDEV_LED_OFF as isize,
}

pub struct DeviceId {
    pub bustype: BusType,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

pub struct AbsInfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}

pub struct Device {
    raw: *mut raw::libevdev,
}

fn ptr_to_str(ptr: *const c_char) -> Option<String> {
    let slice : Option<&CStr> = unsafe {
        if ptr.is_null() {
            return None
        }
        Some(CStr::from_ptr(ptr))
    };

    match slice {
        None => None,
        Some(s) => {
            let buf : &[u8] = s.to_bytes();
            let str_slice: &str = std::str::from_utf8(buf).unwrap();
            Some(str_slice.to_owned())
        }
    }
}

pub fn property_get_name(prop: u32) -> Option<String> {
    ptr_to_str(unsafe {
        raw::libevdev_property_get_name(prop)
    })
}

pub fn event_type_get_name(type_: u32) -> Option<String> {
    ptr_to_str(unsafe {
        raw::libevdev_event_type_get_name(type_)
    })
}

pub fn event_code_get_name(type_: u32, code: u32) -> Option<String> {
    ptr_to_str(unsafe {
        raw::libevdev_event_code_get_name(type_, code)
    })
}

impl Device {
    pub fn new() -> Device {
        let libevdev = unsafe {
            raw::libevdev_new()
        };

        if libevdev.is_null() {
            // FIXME: what to do here?
            panic!("OOM");
        }

        Device {
            raw: libevdev,
        }
    }

    pub fn name(&self) -> String {
        ptr_to_str(unsafe {
            raw::libevdev_get_name(self.raw)
        }).unwrap()
    }

    pub fn uniq(&self) -> Option<String> {
        ptr_to_str(unsafe {
            raw::libevdev_get_uniq(self.raw)
        })
    }

    pub fn phys(&self) -> Option<String> {
        ptr_to_str(unsafe {
            raw::libevdev_get_phys(self.raw)
        })
    }

    pub fn set_fd(&mut self, f: &File) -> Result<(), nix::errno::Errno> {
        let result = unsafe {
            raw::libevdev_set_fd(self.raw, f.as_raw_fd())
        };

        if result == 0 {
            Ok(())
        } else {
            let e = nix::errno::from_i32(-result);
            Err(e)
        }
    }

    pub fn fd(&self) -> Option<File> {
        let result = unsafe {
            raw::libevdev_get_fd(self.raw)
        };

        if result == 0 {
            None
        } else {
            unsafe {
                let f = File::from_raw_fd(result);
                Some(f)
            }
        }
    }

    pub fn change_fd(&mut self, f: &File) -> Result<(), nix::errno::Errno>  {
        let result = unsafe {
            raw::libevdev_change_fd(self.raw, f.as_raw_fd())
        };

        if result == 0 {
            Ok(())
        } else {
            let e = nix::errno::from_i32(-result);
            Err(e)
        }
    }

    pub fn grab(&mut self, grab: GrabMode) -> Result<(), i32> {
        let result = unsafe {
            raw::libevdev_grab(self.raw, grab as i32)
        };

        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn get_abs_info(&self, code: u32) -> Option<AbsInfo> {
        let a = unsafe {
            raw::libevdev_get_abs_info(self.raw, code)
        };

        if a.is_null() {
            return None
        }

        unsafe {
            let absinfo = AbsInfo {
                value: (*a).value,
                minimum: (*a).minimum,
                maximum: (*a).maximum,
                fuzz: (*a).fuzz,
                flat: (*a).flat,
                resolution: (*a).resolution,
            };
            Some(absinfo)
        }
    }

    pub fn has_property(&self, prop: u32) -> bool {
        unsafe {
            raw::libevdev_has_property(self.raw, prop) != 0
        }
    }

    pub fn has_event_type(&self, type_: u32) -> bool {
        unsafe {
            raw::libevdev_has_event_type(self.raw, type_) != 0
        }
    }

    pub fn has_event_code(&self, type_: u32, code: u32) -> bool {
        unsafe {
            raw::libevdev_has_event_code(self.raw, type_, code) != 0
        }
    }

    pub fn get_event_value(&self, type_: u32, code: u32) -> Option<i32> {
        unsafe {
            let mut value :i32 = 0;
            let valid = raw::libevdev_fetch_event_value(self.raw,
                                                        type_,
                                                        code,
                                                        &mut value);
            if valid != 0 {
                Some(value)
            } else {
                None
            }
        }
    }

    pub fn has_event_pending(&self) -> i32 {
        unsafe {
            raw::libevdev_has_event_pending(self.raw)
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            raw::libevdev_free(self.raw);
        }
    }
}

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
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(&f).unwrap();
    match d.name().as_ref() {
        "" => panic!("Invalid name"),
        _ => ..,
    };
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
    let v2 = d.get_event_value(EV::EV_SYN as u32, SYN::SYN_REPORT as u32); // SYN_REPORT
    assert_eq!(v2, Some(0));
}
