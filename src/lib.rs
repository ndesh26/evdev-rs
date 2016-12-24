extern crate evdev_sys as raw;
extern crate nix;
extern crate libc;

pub mod consts;
pub mod log;
#[macro_use]
pub mod macros;

use libc::{c_char, c_int, c_uint};
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::fs::File;
use std::ffi::{CStr, CString};
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

fn ptr_to_str(ptr: *const c_char) -> Option<&'static str> {
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
            Some(std::str::from_utf8(buf).unwrap())
        }
    }
}

pub fn property_get_name(prop: u32) -> Option<&'static str> {
    ptr_to_str(unsafe {
        raw::libevdev_property_get_name(prop)
    })
}

pub fn event_type_get_name(type_: u32) -> Option<&'static str> {
    ptr_to_str(unsafe {
        raw::libevdev_event_type_get_name(type_)
    })
}

pub fn event_code_get_name(type_: u32, code: u32) -> Option<&'static str> {
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

    string_getter!(name, libevdev_get_name,
                   phys, libevdev_get_phys,
                   uniq, libevdev_get_uniq);
    string_setter!(set_name, libevdev_set_name,
                   set_phys, libevdev_set_phys,
                   set_uniq, libevdev_set_uniq);

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
            raw::libevdev_grab(self.raw, grab as c_int)
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
            raw::libevdev_has_property(self.raw, prop as c_uint) != 0
        }
    }

    pub fn enable_property(&self, prop: u32) -> Result<(), nix::errno::Errno> {
        let result = unsafe {
            raw::libevdev_enable_property(self.raw, prop as c_uint) as i32
        };

        if result == 0 {
            Ok(())
        } else {
            let e = nix::errno::from_i32(-result);
            Err(e)
        }
    }

    pub fn has_event_type(&self, type_: u32) -> bool {
        unsafe {
            raw::libevdev_has_event_type(self.raw, type_ as c_uint) != 0
        }
    }

    pub fn has_event_code(&self, type_: u32, code: u32) -> bool {
        unsafe {
            raw::libevdev_has_event_code(self.raw,
                                         type_ as c_uint,
                                         code as c_uint) != 0
        }
    }

    pub fn get_event_value(&self, type_: u32, code: u32) -> Option<i32> {
        unsafe {
            let mut value :i32 = 0;
            let valid = raw::libevdev_fetch_event_value(self.raw,
                                                        type_ as c_uint,
                                                        code as c_uint,
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

    product_getter!(product_id, libevdev_get_id_product,
                    vendor_id, libevdev_get_id_vendor,
                    bustype, libevdev_get_id_bustype,
                    version, libevdev_get_id_version);

    product_setter!(set_product_id, libevdev_set_id_product,
                    set_vendor_id, libevdev_set_id_vendor,
                    set_bustype, libevdev_set_id_bustype,
                    set_version, libevdev_set_id_version);

    pub fn driver_version(&self) -> i32 {
        unsafe {
            raw::libevdev_get_driver_version(self.raw) as i32
        }
    }

    abs_getter!(abs_minimum, libevdev_get_abs_minimum,
                abs_maximum, libevdev_get_abs_maximum,
                abs_fuzz, libevdev_get_abs_fuzz,
                abs_flat, libevdev_get_abs_flat,
                abs_resolution, libevdev_get_abs_resolution);

    abs_setter!(set_abs_minimum, libevdev_set_abs_minimum,
                set_abs_maximum, libevdev_set_abs_maximum,
                set_abs_fuzz, libevdev_set_abs_fuzz,
                set_abs_flat, libevdev_set_abs_flat,
                set_abs_resolution, libevdev_set_abs_resolution);

}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            raw::libevdev_free(self.raw);
        }
    }
}
