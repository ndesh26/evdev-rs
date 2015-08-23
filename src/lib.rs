extern crate libc;

use libc::{c_int};
use std::os::unix::io::AsRawFd;
use std::fs::File;

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

    pub fn set_fd(&mut self, f: File) -> Result<(), i32> {
        let result = unsafe {
            libevdev_set_fd(self.libevdev, f.as_raw_fd())
        };

        self.fd = Some(f);

        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn change_fd(&mut self, f: File) -> Result<(), i32>  {
        let result = unsafe {
            libevdev_change_fd(self.libevdev, f.as_raw_fd())
        };
        self.fd = Some(f);

        if result == 0 {
            Ok(())
        } else {
            Err(result)
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
        Err(result) => panic!("Error {}", result),
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
        Err(result) => panic!("Error {}", result),
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
