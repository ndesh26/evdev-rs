//! Rust bindings to libevdev, an wrapper for evdev devices.
//!
//! This library intends to provide a safe interface to the libevdev library. It
//! will look for the library on the local system, and link to the installed copy.
//!
//! # Examples
//!
//! ## Intializing a evdev device
//!
//! ```
//! use evdev::Device;
//! use std::fs::File;
//!
//! let f = File::open("/dev/input/event0").unwrap();
//!
//! let mut d = Device::new().unwrap();
//! d.set_fd(&f).unwrap();
//! ```
//!
//! ## Getting the next event
//!
//! ```rust,no_run
//! use evdev::Device;
//! use std::fs::File;
//!
//! let f = File::open("/dev/input/event0").unwrap();
//!
//! let mut d = Device::new().unwrap();
//! d.set_fd(&f).unwrap();
//!
//! loop {
//!     let a = d.next_event(evdev::NORMAL | evdev::BLOCKING);
//!     match a {
//!         Ok(k) => println!("Event: time {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
//!				              k.1.time.tv_sec,
//!				              k.1.time.tv_usec,
//!				              evdev::event_type_get_name(k.1.event_type).unwrap()),
//!         Err(e) => (),
//!     }
//! }
//! ```

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
use consts::*;

#[derive(Copy)]
#[derive(Clone)]
pub enum BusType {
    USB,
}

pub enum GrabMode {
    /// Grab the device if not currently grabbed
    Grab = raw::LIBEVDEV_GRAB as isize,
    /// Ungrab the device if currently grabbed
    Ungrab = raw::LIBEVDEV_UNGRAB as isize,
}

bitflags! {
    pub flags ReadFlag: u32 {
        /// Process data in sync mode
        const SYNC = 1,
        /// Process data in normal mode
        const NORMAL = 2,
        /// Pretend the next event is a SYN_DROPPED and require the
        /// caller to sync
        const FORCE_SYNC = 4,
        /// The fd is not in O_NONBLOCK and a read may block
        const BLOCKING = 8,
    }
}

#[derive(PartialEq)]
pub enum ReadStatus {
    /// `next_event` has finished without an error and an event is available
    /// for processing.
    Success = raw::LIBEVDEV_READ_STATUS_SUCCESS as isize,
    /// Depending on the `next_event` read flag:
	/// libevdev received a SYN_DROPPED from the device, and the caller should
	/// now resync the device, or, an event has been read in sync mode.
    Sync = raw::LIBEVDEV_READ_STATUS_SYNC as isize,
}

pub enum LedState {
    /// Turn the LED on
    On = raw::LIBEVDEV_LED_ON as isize,
    /// Turn the LED off
    Off = raw::LIBEVDEV_LED_OFF as isize,
}

pub struct DeviceId {
    pub bustype: BusType,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

/// used by EVIOCGABS/EVIOCSABS ioctls
pub struct AbsInfo {
    /// latest reported value for the axis
    pub value: i32,
    /// specifies minimum value for the axis
    pub minimum: i32,
    /// specifies maximum value for the axis
    pub maximum: i32,
    /// specifies fuzz value that is used to filter noise from
    /// the event stream
    pub fuzz: i32,
    /// values that are within this value will be discarded by
    /// joydev interface and reported as 0 instead
    pub flat: i32,
    /// specifies resolution for the values reported for
    /// the axis
    pub resolution: i32,
}

/// Opaque struct representing an evdev device
pub struct Device {
    raw: *mut raw::libevdev,
}

pub struct TimeVal {
   pub tv_sec: i64,
   pub tv_usec: i64,
}

/// The event structure itself
pub struct InputEvent {
    /// The time at which event occured
    pub time: TimeVal,
    pub event_type: EventType,
    pub event_code: EventCode,
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

fn event_code_to_raw(event_code: EventCode) -> (c_uint, c_uint) {
    let mut ev_type: c_uint = 0;
    let mut ev_code: c_uint = 0;
    match event_code {
        EventCode::EV_SYN(code) => {
            ev_type = EventType::EV_SYN as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_KEY(code) => {
            ev_type = EventType::EV_KEY as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_REL(code) => {
            ev_type = EventType::EV_REL as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_ABS(code) => {
            ev_type = EventType::EV_ABS as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_MSC(code) => {
            ev_type = EventType::EV_MSC as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_SW(code) => {
            ev_type = EventType::EV_SW as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_LED(code) => {
            ev_type = EventType::EV_LED as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_SND(code) => {
            ev_type = EventType::EV_SND as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_REP(code) => {
            ev_type = EventType::EV_REP as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_FF(code) => {
            ev_type = EventType::EV_FF as c_uint;
            ev_code = code as c_uint;
        },
        EventCode::EV_FF_STATUS(code) => {
            ev_type = EventType::EV_FF_STATUS as c_uint;
            ev_code = code as c_uint;
        },
        _ => {}
    } 

    (ev_type, ev_code)
    
}

fn event_code_from_raw(event_type: c_uint, event_code: c_uint) -> EventCode {
    let ev_type: EventType = consts::event_type(event_type as u32).unwrap();
    let ev_code: EventCode;
    match ev_type {
        EventType::EV_SYN => ev_code = EventCode::EV_SYN(syn(event_code as u32).unwrap()),
        EventType::EV_KEY => ev_code = EventCode::EV_KEY(key(event_code as u32).unwrap()),
        EventType::EV_ABS => ev_code = EventCode::EV_ABS(abs(event_code as u32).unwrap()),
        EventType::EV_REL => ev_code = EventCode::EV_REL(rel(event_code as u32).unwrap()),
        EventType::EV_MSC => ev_code = EventCode::EV_MSC(msc(event_code as u32).unwrap()),
        EventType::EV_SW => ev_code = EventCode::EV_SW(sw(event_code as u32).unwrap()),
        EventType::EV_LED => ev_code = EventCode::EV_LED(led(event_code as u32).unwrap()),
        EventType::EV_SND => ev_code = EventCode::EV_SND(snd(event_code as u32).unwrap()),
        EventType::EV_REP => ev_code = EventCode::EV_REP(rep(event_code as u32).unwrap()),
        EventType::EV_FF => ev_code = EventCode::EV_FF(ff(event_code as u32).unwrap()),
        EventType::EV_PWR => ev_code = EventCode::EV_PWR,
        EventType::EV_FF_STATUS => ev_code = EventCode::EV_FF_STATUS(ff(event_code as u32).unwrap()),
        EventType::EV_MAX => ev_code = EventCode::EV_MAX,
    }

    ev_code
}

/// The name of the given input prop (e.g. INPUT_PROP_BUTTONPAD) or None for an
/// invalid property
pub fn property_get_name(prop: u32) -> Option<&'static str> {
    ptr_to_str(unsafe {
        raw::libevdev_property_get_name(prop)
    })
}

/// The name of the given event type (e.g. EV_ABS) or None for an
/// invalid type
pub fn event_type_get_name(event_type: EventType) -> Option<&'static str> {
    ptr_to_str(unsafe {
        raw::libevdev_event_type_get_name(event_type as c_uint)
    })
}

/// The name of the given event code (e.g. ABS_X) or None for an
/// invalid type or code
pub fn event_code_get_name(event_code: EventCode) -> Option<&'static str> {
    let (ev_type, ev_code) = event_code_to_raw(event_code);
    ptr_to_str(unsafe {
        raw::libevdev_event_code_get_name(ev_type, ev_code)
    })
}

/// The given type constant for the passed name or Errno if not found.
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

/// Look up an event code by its type and name. Event codes start with a fixed
/// prefix followed by their name (eg., "ABS_X"). The prefix must be included in
/// the name. It returns the constant assigned to the event code or Errno if not
/// found.
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

/// Look up an input property by its name. Properties start with the fixed
/// prefix "INPUT_PROP_" followed by their name (eg., "INPUT_PROP_POINTER").
/// The prefix must be included in the name. It returns the constant assigned
/// to the property or Errno if not found.
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

/// The max value defined for the given event type, e.g. ABS_MAX for a type
/// of EV_ABS, or Errno for an invalid type.
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

    /// Change the abs info for the given EV_ABS event code, if the code exists.
    ///
    /// This function has no effect if `has_event_code` returns false for
    /// this code.
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
    pub fn has_event_type(&self, ev_type: EventType) -> bool {
        unsafe {
            raw::libevdev_has_event_type(self.raw, ev_type as c_uint) != 0
        }
    }

    /// Return `true` is the device support this event type and code and `false` otherwise
    pub fn has_event_code(&self, code: EventCode) -> bool {
        unsafe {
            let (ev_type, ev_code) = event_code_to_raw(code);
            raw::libevdev_has_event_code(self.raw,
                                         ev_type,
                                         ev_code) != 0
        }
    }

    ///  Returns the current value of the event type.
    ///
    /// If the device supports this event type and code, the return value is
    /// set to the current value of this axis. Otherwise, `None` is returned.
    pub fn event_value(&self, code: EventCode) -> Option<i32> {
        let mut value: i32 = 0;
        let (ev_type, ev_code) = event_code_to_raw(code);
        let valid = unsafe {
            raw::libevdev_fetch_event_value(self.raw,
                                            ev_type,
                                            ev_code ,
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

    /// Return the current value of the code for the given slot.
    ///
    /// If the device supports this event code, the return value is
    /// is set to the current value of this axis. Otherwise, or
    /// if the event code is not an ABS_MT_* event code, `None` is returned
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

    /// Set the value for a given code for the given slot.
    ///
    /// This is a local modification only affecting only this representation of
    /// this device. A future call to `slot_value` will return this value,
    /// unless the value was overwritten by an event.
    ///
    /// This function does not set event values for axes outside the ABS_MT range,
    /// use `set_event_value` instead.
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

    /// Get the number of slots supported by this device.
    ///
    /// The number of slots supported, or `None` if the device does not provide
    /// any slots
    ///
    /// A device may provide ABS_MT_SLOT but a total number of 0 slots. Hence
    /// the return value of `None` for "device does not provide slots at all"
    pub fn num_slots(&self) -> Option<i32> {
        let result = unsafe {
            raw::libevdev_get_num_slots(self.raw)
        };

        match result  {
            -1 => None,
             k => Some(k),
        }
    }

    /// Get the currently active slot.
    ///
    /// This may differ from the value an ioctl may return at this time as
    /// events may have been read off the fd since changing the slot value
    /// but those events are still in the buffer waiting to be processed.
    /// The returned value is the value a caller would see if it were to
    /// process events manually one-by-one.
    pub fn current_slot(&self) -> Option<i32> {
        let result = unsafe {
            raw::libevdev_get_current_slot(self.raw)
        };

        match result {
            -1 => None,
             k => Some(k),
        }
    }

    /// Forcibly enable an event type on this device, even if the underlying
    /// device does not support it. While this cannot make the device actually
    /// report such events, it will now return true for libevdev_has_event_type().
    ///
    /// This is a local modification only affecting only this representation of
    /// this device.
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

    /// Forcibly disable an event type on this device, even if the underlying
    /// device provides it. This effectively mutes the respective set of
    /// events. libevdev will filter any events matching this type and none will
    /// reach the caller. libevdev_has_event_type() will return false for this
    /// type.
    ///
    /// In most cases, a caller likely only wants to disable a single code, not
    /// the whole type. Use `disable_event_code` for that.
    ///
    /// Disabling EV_SYN will not work. In Peter's Words "Don't shoot yourself
    /// in the foot. It hurts".
    ///
    /// This is a local modification only affecting only this representation of
    /// this device.
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
    /// Forcibly disable an event code on this device, even if the underlying
    /// device provides it. This effectively mutes the respective set of
    /// events. libevdev will filter any events matching this type and code and
    /// none will reach the caller. `has_event_code` will return false for
    /// this code.
    ///
    /// Disabling all event codes for a given type will not disable the event
    /// type. Use `disable_event_type` for that.
    ///
    /// This is a local modification only affecting only this representation of
    /// this device.
    ///
    /// Disabling codes of type EV_SYN will not work. Don't shoot yourself in the
    /// foot. It hurts.
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

    /// Set the device's EV_ABS axis to the value defined in the abs
    /// parameter. This will be written to the kernel.
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

    /// Turn an LED on or off.
    ///
    /// enabling an LED requires write permissions on the device's file descriptor.
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

    /// Set the clock ID to be used for timestamps. Further events from this device
    /// will report an event time based on the given clock.
    ///
    /// This is a modification only affecting this representation of
    /// this device.
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
            event_type: 0,
            event_code: 0,
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
            event_type: event_type(ev.event_type as u32).unwrap(),
            event_code: event_code_from_raw(ev.event_type as u32, ev.event_code as u32),
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
            event_type: self.event_type as u16,
            event_code: event_code_to_raw(self.event_code).1 as u16,
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
            event_type: self.event_type as u16,
            event_code: event_code_to_raw(self.event_code).1 as u16,
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
