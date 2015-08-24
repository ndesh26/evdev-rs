extern crate libc;
extern crate nix;

use libc::{c_int, c_char};
use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::ffi::CStr;

#[repr(C)]
struct Libevdev;

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
