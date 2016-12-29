extern crate evdev_sys as raw;
extern crate nix;
extern crate libc;
#[macro_use]
extern crate bitflags;

pub mod consts;
pub mod log;
#[macro_use]
mod macros;

use libc::{c_char, c_int, c_uint};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::File;
use std::ffi::{CStr, CString};
use nix::errno::Errno;

#[derive(Copy)]
#[derive(Clone)]
pub enum BusType {
    USB,
}

pub enum GrabMode {
    Grab = raw::LIBEVDEV_GRAB as isize,
    Ungrab = raw::LIBEVDEV_UNGRAB as isize,
}

bitflags! {
    pub flags ReadFlag: u32 {
        const SYNC = 1,
        const NORMAL = 2,
        const FORCE_SYNC = 4,
        const BLOCKING = 8,
    }
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

pub struct TimeVal {
   pub tv_sec: i64,
   pub tv_usec: i64,
}

pub struct InputEvent {
    pub time: TimeVal,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
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
    /// This function only initializesthe struct to sane default values.
    /// To actually hook up the device to a kernel device, use `set_fd`.
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
    /// use evdev::Device;
    /// # use std::fs::File;
    ///
    /// let mut device = Device::new().unwrap();
    /// # let fd = File::open("/dev/input/event0").unwrap();
    /// device.set_fd(&fd);
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
    /// This function may only be called once per device. If the device changed and
    /// you need to re-read a device, use `new` method. If you need to change the file after
    /// closing and re-opening the same device, use `change_fd`.
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

    /// Change the fd for this device, without re-reading the actual device.
    ///
    /// If the fd changes after initializing the device, for example after a
    /// VT-switch in the X.org X server, this function updates the internal fd
    /// to the newly opened. No check is made that new fd points to the same
    /// device. If the device has changed, evdev's behavior is undefined.
    ///
    /// evdev device does not sync itself after changing the fd and keeps the current
    /// device state. Use next_event with the FORCE_SYNC flag to force a re-sync.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// dev.change_fd(new_fd);
    /// dev.next_event(evdev::FORCE_SYNC);
    /// while dev.next_event(evdev::SYNC).ok().unwrap().0 == ReadStatus::SYNC
    ///                             {} // noop
    /// ```
    /// It is an error to call this function before calling set_fd().
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

    /// Get the axis info for the given axis, as advertised by the kernel.
    ///
    /// Returns the `AbsInfo` for the given the code or None if the device
    /// doesn't support this code
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

    /// Return `true` if device support the property and false otherwise
    pub fn has_property(&self, prop: u32) -> bool {
        unsafe {
            raw::libevdev_has_property(self.raw, prop as c_uint) != 0
        }
    }

    /// Enables this property, a call to `set_fd` will overwrite any previously set values
    pub fn enable_property(&self, prop: u32) -> Result<(), Errno> {
        let result = unsafe {
            raw::libevdev_enable_property(self.raw, prop as c_uint) as i32
        };

        match result {
            0 => Ok(()),
            k => Err(Errno::from_i32(-k))
        }
    }
    /// Returns `true` is the device support this event type and `false` otherwise
    pub fn has_event_type(&self, type_: u32) -> bool {
        unsafe {
            raw::libevdev_has_event_type(self.raw, type_ as c_uint) != 0
        }
    }

    /// Return `true` is the device support this event type and code and `false` otherwise
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

    /// Set the value for a given event type and code.
    ///
    /// This only makes sense for some event types, e.g. setting the value for
    /// EV_REL is pointless.
    ///
    /// This is a local modification only affecting only this representation of
    /// this device. A future call to libevdev_get_event_value() will return this
    /// value, unless the value was overwritten by an event.
    ///
    /// If the device supports ABS_MT_SLOT, the value set for any ABS_MT_*
    /// event code is the value of the currently active slot. You should use
    /// `set_slot_value` instead.
    ///
    /// If the device supports ABS_MT_SLOT and the type is EV_ABS and the code is
    /// ABS_MT_SLOT, the value must be a positive number less then the number of
    /// slots on the device. Otherwise, `set_event_value` returns Err.
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

    /// Return the driver version of a device already intialize with `set_fd`
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

    /// Get the next event from the device. This function operates in two different
    /// modes: normal mode or sync mode.
    ///
    /// In normal mode (when flags has `evdev::NORMAL` set), this function returns
    /// `ReadStatus::Success` and returns the event. If no events are available at
    /// this time, it returns `-EAGAIN` as `Err`.
    ///
    /// If the current event is an `EV_SYN::SYN_DROPPED` event, this function returns
    /// `ReadStatus::Sync` and is set to the `EV_SYN` event.The caller should now call
    /// this function with the `evdev::SYNC` flag set, to get the set of events that
    /// make up the device state delta. This function returns ReadStatus::Sync for
    /// each event part of that delta, until it returns `-EAGAIN` once all events
    /// have been synced.
    ///
    /// If a device needs to be synced by the caller but the caller does not call
    /// with the `evdev::SYNC` flag set, all events from the diff are dropped after
    /// evdev updates its internal state and event processing continues as normal.
    /// Note that the current slot and the state of touch points may have updated
    /// during the `SYN_DROPPED` event, it is strongly recommended that a caller
    /// ignoring all sync events calls `get_current_slot` and checks the
    /// `ABS_MT_TRACKING_ID` values for all slots.
    ///
    /// If a device has changed state without events being enqueued in evdev,
    /// e.g. after changing the file descriptor, use the `evdev::FORCE_SYNC` flag.
    /// This triggers an internal sync of the device and `next_event` returns
    /// `ReadStatus::Sync`.

    pub fn next_event(&self, flags: ReadFlag)
                      -> Result<(ReadStatus, InputEvent), Errno> {
        let mut ev = raw::input_event {
            time: raw::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: 0,
            code: 0,
            value: 0,
        };

        let result = unsafe {
            raw::libevdev_next_event(self.raw, flags.bits as c_uint, &mut ev)
        };

        let event = InputEvent {
            time: TimeVal {
                tv_sec: ev.time.tv_sec,
                tv_usec: ev.time.tv_usec,
            },
            type_: ev.type_,
            code: ev.code,
            value: ev.value,
        };

        match result {
            raw::LIBEVDEV_READ_STATUS_SUCCESS => Ok((ReadStatus::Success, event)),
            raw::LIBEVDEV_READ_STATUS_SYNC => Ok((ReadStatus::Sync, event)),
            k => Err(Errno::from_i32(-k)),
        }
    }
}

impl InputEvent {
    pub fn is_type(&self, type_: u16) -> bool {
        let ev = raw::input_event {
            time: raw::timeval {
                tv_sec: self.time.tv_sec,
                tv_usec: self.time.tv_usec,
            },
            type_: self.type_,
            code: self.code,
            value: self.value,
        };

        unsafe {
            raw::libevdev_event_is_type(&ev, type_ as c_uint) == 1
        }
    }

    pub fn is_code(&self, type_: u16, code: u16) -> bool {
        let ev = raw::input_event {
            time: raw::timeval {
                tv_sec: self.time.tv_sec,
                tv_usec: self.time.tv_usec,
            },
            type_: self.type_,
            code: self.code,
            value: self.value,
        };

        unsafe {
            raw::libevdev_event_is_code(&ev, type_ as c_uint, code as c_uint) == 1
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
