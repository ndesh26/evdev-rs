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
//! use evdev_rs::Device;
//! use std::fs::File;
//!
//! let f = File::open("/dev/input/event0").unwrap();
//!
//! let mut d = Device::new().unwrap();
//! d.set_fd(f).unwrap();
//! ```
//!
//! ## Getting the next event
//!
//! ```rust,no_run
//! use evdev_rs::Device;
//! use std::fs::File;
//!
//! let f = File::open("/dev/input/event0").unwrap();
//!
//! let mut d = Device::new().unwrap();
//! d.set_fd(f).unwrap();
//!
//! loop {
//!     let a = d.next_event(evdev_rs::NORMAL | evdev_rs::BLOCKING);
//!     match a {
//!         Ok(k) => println!("Event: time {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
//!				              k.1.time.tv_sec,
//!				              k.1.time.tv_usec,
//!				              k.1.event_type),
//!         Err(e) => (),
//!     }
//! }
//! ```

extern crate evdev_sys as raw;
extern crate nix;
extern crate libc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

#[macro_use]
mod macros;
pub mod device;
pub mod enums;
pub mod logging;
pub mod uinput;
pub mod util;

use libc::{c_long, c_uint};

use enums::*;
use util::*;

#[doc(inline)]
pub use device::Device;
#[doc(inline)]
pub use uinput::UInputDevice;

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

impl AbsInfo {
    pub fn from_raw(absinfo: libc::input_absinfo) -> AbsInfo {
        AbsInfo {
            value: absinfo.value,
            minimum: absinfo.minimum,
            maximum: absinfo.maximum,
            fuzz: absinfo.fuzz,
            flat: absinfo.flat,
            resolution: absinfo.resolution,
        }
    }

    pub fn as_raw(&self) -> libc::input_absinfo {
        libc::input_absinfo {
            value: self.value,
            minimum: self.minimum,
            maximum: self.maximum,
            fuzz: self.fuzz,
            flat: self.flat,
            resolution: self.resolution,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimeVal {
   pub tv_sec: c_long,
   pub tv_usec: c_long,
}

impl TimeVal {
    pub fn new(tv_sec: i64, tv_usec: i64) -> TimeVal {
        TimeVal { tv_sec, tv_usec }
    }

    pub fn from_raw(timeval: &libc::timeval) -> TimeVal {
        TimeVal {
            tv_sec: timeval.tv_sec,
            tv_usec: timeval.tv_usec,
        }
    }

    pub fn as_raw(&self) -> libc::timeval {
        libc::timeval {
            tv_sec: self.tv_sec,
            tv_usec: self.tv_usec
        }
    }
}

/// The event structure itself
#[derive(Clone, Debug, PartialEq)]
pub struct InputEvent {
    /// The time at which event occured
    pub time: TimeVal,
    pub event_type: EventType,
    pub event_code: EventCode,
    pub value: i32,
}

impl InputEvent {
    pub fn new(timeval: &TimeVal, code: &EventCode, value: i32) -> InputEvent {
        let (ev_type, _) = event_code_to_int(&code);
        InputEvent {
            time: timeval.clone(),
            event_type: int_to_event_type(ev_type).unwrap(),
            event_code: code.clone(),
            value
        }
    }

    pub fn from_raw(event: &libc::input_event) -> InputEvent {
        let ev_type = event.type_ as u32;
        let event_type = int_to_event_type(ev_type).unwrap();
        let event_code = int_to_event_code(ev_type, event.code as u32).unwrap();
        InputEvent {
            time: TimeVal::from_raw(&event.time),
            event_type,
            event_code,
            value: event.value
        }
    }

    pub fn as_raw(&self) -> libc::input_event {
        let (ev_type, ev_code) = event_code_to_int(&self.event_code);
        libc::input_event {
            time: self.time.as_raw(),
            type_: ev_type as u16,
            code: ev_code as u16,
            value: self.value
        }
    }

    pub fn is_type(&self, ev_type: &EventType) -> bool {
        unsafe {
            raw::libevdev_event_is_type(&self.as_raw(), ev_type.clone() as c_uint) == 1
        }
    }

    pub fn is_code(&self, code: &EventCode) -> bool {
        let (ev_type, ev_code) = event_code_to_int(code);

        unsafe {
            raw::libevdev_event_is_code(&self.as_raw(), ev_type, ev_code) == 1
        }
    }
}
