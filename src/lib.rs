extern crate libc;
extern crate nix;

use libc::{c_int, c_uint, c_char, int32_t};
use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::ffi::CStr;

#[repr(C)]
struct Libevdev;

#[repr(C)]
struct input_absinfo {
    value: int32_t,
    minimum: int32_t,
    maximum: int32_t,
    fuzz: int32_t,
    flat: int32_t,
    resolution: int32_t,
}

#[link(name = "evdev")]
extern {
    fn libevdev_new() -> *mut Libevdev;
    fn libevdev_free(ctx: *mut Libevdev);
    fn libevdev_set_fd(ctx: *mut Libevdev, fd: c_int) -> c_int;
    fn libevdev_change_fd(ctx: *mut Libevdev, fd: c_int) -> c_int;
    fn libevdev_get_fd(ctx: *mut Libevdev) -> c_int;
    fn libevdev_grab(ctx: *mut Libevdev, grab: c_int) -> c_int;
    fn libevdev_get_name(ctx: *const Libevdev) -> *const c_char;
    fn libevdev_get_uniq(ctx: *const Libevdev) -> *const c_char;
    fn libevdev_get_phys(ctx: *const Libevdev) -> *const c_char;
    fn libevdev_get_abs_info(ctx: *const Libevdev, code: c_uint) -> *const input_absinfo;
    fn libevdev_has_property(ctx: *const Libevdev, prop: c_uint) -> c_int;
    fn libevdev_has_event_type(ctx: *const Libevdev, type_: c_uint) -> c_int;
    fn libevdev_has_event_code(ctx: *const Libevdev, type_: c_uint, code: c_uint) -> c_int;
    fn libevdev_fetch_event_value(ctx: *const Libevdev,
                                  type_: c_uint,
                                  code: c_uint,
                                  value: *mut c_int) -> c_int;
}

#[derive(Copy)]
#[derive(Clone)]
pub enum BusType {
    USB,
}

pub enum GrabMode {
    GRAB,
    UNGRAB,
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
    name: String,
    phys: Option<String>,
    uniq: Option<String>,
    id: DeviceId,

    libevdev: *mut Libevdev,
    fd: Option<File>,
}

impl Device {
    pub fn new() -> Device {
        let libevdev = unsafe {
            libevdev_new()
        };

        if libevdev.is_null() {
            // FIXME: what to do here?
            panic!("OOM");
        }

        Device {
            name: String::new(),
            phys: None,
            uniq: None,
            id: DeviceId {
                bustype: BusType::USB,
                vendor: 0,
                product: 0,
                version: 0
            },
            libevdev: libevdev,
            fd: None
        }
    }

    pub fn name(self) -> String {
        self.name.clone()
    }

    pub fn uniq(self) -> Option<String> {
        self.uniq.clone()
    }

    pub fn phys(self) -> Option<String> {
        self.phys.clone()
    }

    pub fn id(self) -> DeviceId {
        DeviceId { 
            bustype: self.id.bustype,
            vendor: self.id.vendor,
            product: self.id.product,
            version: self.id.version,
        }
    }

    fn ptr_to_str(&self, ptr: *const c_char) -> Option<String> {
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

    fn update(&mut self) {
        // libevdev guarantees name is not NULL
        self.name = self.ptr_to_str(unsafe {
            libevdev_get_name(self.libevdev)
        }).unwrap();

        self.uniq = self.ptr_to_str(unsafe {
            libevdev_get_uniq(self.libevdev)
        });

        self.phys = self.ptr_to_str(unsafe {
            libevdev_get_phys(self.libevdev)
        });
    }

    pub fn set_fd(&mut self, f: File) -> Result<(), nix::errno::Errno> {
        let result = unsafe {
            libevdev_set_fd(self.libevdev, f.as_raw_fd())
        };

        self.fd = Some(f);

        if result == 0 {
            self.update();
            Ok(())
        } else {
            let e = nix::errno::from_i32(-result);
            Err(e)
        }
    }

    pub fn change_fd(&mut self, f: File) -> Result<(), nix::errno::Errno>  {
        let result = unsafe {
            libevdev_change_fd(self.libevdev, f.as_raw_fd())
        };
        self.fd = Some(f);

        if result == 0 {
            Ok(())
        } else {
            let e = nix::errno::from_i32(-result);
            Err(e)
        }
    }

    pub fn get_fd(self) -> Option<File> {
        // FIXME: Drop trait prevents self.fd from being returned, and
        // there's no clone() for File
        // Not sure how this will fit with rust's ownership handling anyway
        None
    }

    pub fn grab(&mut self, grab: GrabMode) -> Result<(), i32> {
        let result = unsafe {
            let mode = match grab {
                GrabMode::GRAB => 3,
                GrabMode::UNGRAB => 4,
            };
            libevdev_grab(self.libevdev, mode)
        };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn get_abs_info(&self, code: u32) -> Option<AbsInfo> {
        let a = unsafe {
            libevdev_get_abs_info(self.libevdev, code)
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
            libevdev_has_property(self.libevdev, prop) != 0
        }
    }

    pub fn has_event_type(&self, type_: u32) -> bool {
        unsafe {
            libevdev_has_event_type(self.libevdev, type_) != 0
        }
    }

    pub fn has_event_code(&self, type_: u32, code: u32) -> bool {
        unsafe {
            libevdev_has_event_code(self.libevdev, type_, code) != 0
        }
    }

    pub fn get_event_value(&self, type_: u32, code: u32) -> Option<i32> {
        unsafe {
            let mut value :i32 = 0;
            let valid = libevdev_fetch_event_value(self.libevdev,
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
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            libevdev_free(self.libevdev);
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

    match d.set_fd(f) {
        Ok(()) => ..,
        Err(result) => panic!("Error {}", result.desc()),
    };
}

#[test]
fn context_change_fd() {
    let mut d = Device::new();
    let f1 = File::open("/dev/input/event0").unwrap();
    let f2 = File::open("/dev/input/event0").unwrap();

    d.set_fd(f1).unwrap();
    match d.change_fd(f2) {
        Ok(()) => ..,
        Err(result) => panic!("Error {}", result.desc()),
    };
}

#[test]
fn context_grab() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(f).unwrap();
    d.grab(GrabMode::GRAB).unwrap();
    d.grab(GrabMode::UNGRAB).unwrap();
}

#[test]
fn device_get_name() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(f).unwrap();
    match d.name().as_ref() {
        "" => panic!("Invalid name"),
        _ => ..,
    };
}

#[test]
fn device_get_uniq() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(f).unwrap();
    match d.uniq() {
        _ => ..,
    };
}

#[test]
fn device_get_phys() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(f).unwrap();
    match d.phys() {
        _ => ..,
    };
}

#[test]
fn device_get_absinfo() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(f).unwrap();
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

    d.set_fd(f).unwrap();
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

    d.set_fd(f).unwrap();
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

    d.set_fd(f).unwrap();

    assert!(d.has_event_type(0)); // EV_SYN
    assert!(d.has_event_code(0, 0)); // SYN_REPORT
}

#[test]
fn device_get_value() {
    let mut d = Device::new();
    let f = File::open("/dev/input/event0").unwrap();

    d.set_fd(f).unwrap();

    let v1 = d.get_event_value(0xff, 0xff); // garbage
    assert_eq!(v1, None);
    let v2 = d.get_event_value(0x00, 0x00); // SYN_REPORT
    assert_eq!(v2, Some(0));
}
