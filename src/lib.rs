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
use nix::errno::Errno;
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

pub fn event_type_from_name(name: &str) -> Result<i32, Errno> {
    let name = CString::new(name).unwrap();
    let result = unsafe {
        raw::libevdev_event_type_from_name(name.as_ptr())
    };

    match result {
        -1 => Err(Errno::from_i32(1)),
         k => Ok(k),
    }
}

pub fn event_code_from_name(type_: u32, name: &str) -> Result<i32, Errno> {
    let name = CString::new(name).unwrap();
    let result = unsafe {
        raw::libevdev_event_code_from_name(type_ as c_uint, name.as_ptr())
    };

    match result {
        -1 => Err(Errno::from_i32(1)),
         k => Ok(k),
    }
}

pub fn property_from_name(name: &str) -> Result<i32, Errno> {
    let name = CString::new(name).unwrap();
    let result = unsafe {
        raw::libevdev_property_from_name(name.as_ptr())
    };

    match result {
        -1 => Err(Errno::from_i32(1)),
         k => Ok(k),
    }
}

pub fn event_type_get_max(type_: u32) -> Result<i32, Errno> {
    let result = unsafe {
        raw::libevdev_event_type_get_max(type_ as c_uint)
    };

    match result {
        -1 => Err(Errno::from_i32(1)),
         k => Ok(k),
    }
}

impl Device {
    /// Initialize a new libevdev device.
    ///
    /// This function only allocates the required memory and initializes
    /// the struct to sane default values. To actually hook up the device
    /// to a kernel device, use `set_fd`.
    pub fn new() -> Option<Device> {
        let libevdev = unsafe {
            raw::libevdev_new()
        };

        if libevdev.is_null() {
            None
        } else {
            Some(Device {
                raw: libevdev,
            })
        }
    }

    /// Initialize a new libevdev device from the given fd.
    ///
    /// This is a shortcut for
    ///
    /// ```
    /// let device = Device::new();
    /// device.set_fd(dev, fd);
    /// ```
    pub fn new_from_fd(fd: &File) -> Result<Device, Errno> {
        let mut libevdev = 0 as *mut _;
        let result = unsafe {
            raw::libevdev_new_from_fd(fd.as_raw_fd(), &mut libevdev)
        };

        match result {
            0 => Ok(Device { raw: libevdev }),
            k => Err(Errno::from_i32(-k)),
        }
    }

    string_getter!(name, libevdev_get_name,
                   phys, libevdev_get_phys,
                   uniq, libevdev_get_uniq);
    string_setter!(set_name, libevdev_set_name,
                   set_phys, libevdev_set_phys,
                   set_uniq, libevdev_set_uniq);

    /// Returns the file associated with the device
    ///
    /// if the `set_fd` hasn't been called yet then it return `None`
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

    /// Set the file for this struct and initialize internal data.
    ///
    /// The file must be opened in O_RDONLY or O_RDWR mode.
    /// This function may only be called once per device. If the device changed and
    /// you need to re-read a device, use `new` method. If you need to change the file after
    /// closing and re-opening the same device, use `change_fd`.
    ///
    /// A caller should ensure that any events currently pending on the fd are
    /// drained before the refrence to the file is passed to evdev for
    /// initialization. Due to how the kernel's ioctl handling works, the initial
    /// device state will reflect the current device state *after* applying all
    /// events currently pending on the fd. Thus, if the fd is not drained, the
    /// state visible to the caller will be inconsistent with the events
    /// immediately available on the device. This does not affect state-less
    /// events like EV_REL.
    ///
    /// Unless otherwise specified, evdev function behavior is undefined until
    /// a successfull call to `set_fd`.
    pub fn set_fd(&mut self, f: &File) -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_set_fd(self.raw, f.as_raw_fd())
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    /// TODO: change the document
    /// Change the fd for this device, without re-reading the actual device.
    ///
    /// If the fd changes after initializing the device, for example after a
    /// VT-switch in the X.org X server, this function updates the internal fd
    /// to the newly opened. No check is made that new fd points to the same
    /// device. If the device has changed, evdev's behavior is undefined.
    ///
    /// evdev device does not sync itself after changing the fd and keeps the current
    /// device state. Use next_event with the LIBEVDEV_READ_FLAG_FORCE_SYNC flag to
    /// force a re-sync.
    ///
    /// The example code below illustrates how to force a re-sync of the
    /// library-internal state. Note that this code doesn't handle the events in
    /// the caller, it merely forces an update of the internal library state.
    /// @code
    ///     struct input_event ev;
    ///     libevdev_change_fd(dev, new_fd);
    ///     libevdev_next_event(dev, LIBEVDEV_READ_FLAG_FORCE_SYNC, &ev);
    ///     while (libevdev_next_event(dev, LIBEVDEV_READ_FLAG_SYNC, &ev) == LIBEVDEV_READ_STATUS_SYNC)
    ///                             ; // noop
    /// @endcode
    /// The fd may be open in O_RDONLY or O_RDWR.
    /// It is an error to call this function before calling libevdev_set_fd().

    pub fn change_fd(&mut self, f: &File) -> Result<(), Errno>  {
        let result = unsafe {
            raw::libevdev_change_fd(self.raw, f.as_raw_fd())
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    /// Grab or ungrab the device through a kernel EVIOCGRAB.
    ///
    /// This prevents other clients (including kernel-internal ones such as
    /// rfkill) from receiving events from this device. This is generally a
    /// bad idea. Don't do this.Grabbing an already grabbed device, or
    /// ungrabbing an ungrabbed device is a noop and always succeeds.
    pub fn grab(&mut self, grab: GrabMode) -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_grab(self.raw, grab as c_int)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k)),
        }
    }

    pub fn abs_info(&self, code: u32) -> Option<AbsInfo> {
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

    pub fn set_abs_info(&self, code: u32, absinfo: &AbsInfo) {
        let absinfo = raw::input_absinfo {
                        value: absinfo.value,
                        minimum: absinfo.minimum,
                        maximum: absinfo.maximum,
                        fuzz: absinfo.fuzz,
                        flat: absinfo.flat,
                        resolution: absinfo.resolution,
                      };

        unsafe {
            raw::libevdev_set_abs_info(self.raw, code as c_uint,
                                       &absinfo as *const _);
        }
    }

    pub fn has_property(&self, prop: u32) -> bool {
        unsafe {
            raw::libevdev_has_property(self.raw, prop as c_uint) != 0
        }
    }

    pub fn enable_property(&self, prop: u32) -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_enable_property(self.raw, prop as c_uint) as i32
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
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

    pub fn event_value(&self, type_: u32, code: u32) -> Option<i32> {
        let mut value: i32 = 0;
        let valid = unsafe {
            raw::libevdev_fetch_event_value(self.raw,
                                            type_ as c_uint,
                                            code as c_uint,
                                            &mut value)
        };

        match valid {
            0 => None,
            _ => Some(value),
        }
    }

    pub fn set_event_value(&self, type_: u32, code: u32, val: i32)
                           -> Result<(), Errno> {
            let result = unsafe {
                raw::libevdev_set_event_value(self.raw,
                                              type_ as c_uint,
                                              code as c_uint,
                                              val as c_int)
            };

            match result {
               0 => Ok(()),
               k => Err(Errno::from_i32(-k))
            }
    }

    /// Check if there are events waiting for us.
    ///
    /// This function does not read an event off the fd and may not access the
    /// fd at all. If there are events queued internally this function will
    /// return non-zero. If the internal queue is empty, this function will poll
    /// the file descriptor for data.
    ///
    /// This is a convenience function for simple processes, most complex programs
    /// are expected to use select(2) or poll(2) on the file descriptor. The kernel
    /// guarantees that if data is available, it is a multiple of sizeof(struct
    /// input_event), and thus calling `next_event` when select(2) or
    /// poll(2) return is safe. You do not need `has_event_pending` if
    /// you're using select(2) or poll(2).
    pub fn has_event_pending(&self) -> bool {
        unsafe {
            raw::libevdev_has_event_pending(self.raw) > 0
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

    pub fn slot_value(&self, slot: u32, code: u32) -> Option<i32> {
        let mut value: i32 = 0;
        let valid = unsafe {
            raw::libevdev_fetch_slot_value(self.raw,
                                           slot as c_uint,
                                           code as c_uint,
                                           &mut value)
        };

        match valid {
            0 => None,
            _ => Some(value),
        }
    }

    pub fn set_slot_value(&self, slot: u32, code: u32, val: i32)
                          -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_set_slot_value(self.raw,
                                         slot as c_uint,
                                         code as c_uint,
                                         val as c_int)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    pub fn num_slots(&self) -> Option<i32> {
        let result = unsafe {
            raw::libevdev_get_num_slots(self.raw)
        };

        match result  {
            -1 => None,
             k => Some(k),
        }
    }

    pub fn current_slot(&self) -> Option<i32> {
        let result = unsafe {
            raw::libevdev_get_current_slot(self.raw)
        };

        match result {
            -1 => None,
             k => Some(k),
        }
    }

    pub fn enable_event_type(&self, type_: u32) -> Result<(), Errno> {
         let result = unsafe {
            raw::libevdev_enable_event_type(self.raw,
                                            type_ as c_uint)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    pub fn disable_event_type(&self, type_: u32) -> Result<(), Errno> {
         let result = unsafe {
            raw::libevdev_disable_event_type(self.raw,
                                            type_ as c_uint)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    pub fn disable_event_code(&self, type_: u32, code: u32)
                              -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_disable_event_code(self.raw,
                                            type_ as c_uint,
                                            code as c_uint)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    pub fn set_kernel_abs_info(&self, code: u32, absinfo: &AbsInfo) {
        let absinfo = raw::input_absinfo {
                        value: absinfo.value,
                        minimum: absinfo.minimum,
                        maximum: absinfo.maximum,
                        fuzz: absinfo.fuzz,
                        flat: absinfo.flat,
                        resolution: absinfo.resolution,
                      };

        unsafe {
            raw::libevdev_kernel_set_abs_info(self.raw, code as c_uint,
                                              &absinfo as *const _);
        }
    }

    pub fn kernel_set_led_value(&self, code: u32, value: LedState)
                                 -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_kernel_set_led_value(self.raw,
                                               code as c_uint,
                                               value as c_int)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }

    pub fn set_clock_id(&self, clockid: i32) -> Result<(), Errno> {
         let result = unsafe {
            raw::libevdev_set_clock_id(self.raw,
                                       clockid as c_int)
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
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
