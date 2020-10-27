//! Rust bindings to libevdev, an wrapper for evdev devices.
//!
//! This library intends to provide a safe interface to the libevdev library. It
//! will look for the library on the local system, and link to the installed copy.
//!
//! # Examples
//!
//! ## Intializing a evdev device
//!
//! ```rust,no_run
//! use evdev_rs::Device;
//! use std::fs::File;
//!
//! let file = File::open("/dev/input/event0").unwrap();
//! let mut d = Device::new_from_file(file).unwrap();
//! ```
//!
//! ## Getting the next event
//!
//! ```rust,no_run
//! use evdev_rs::Device;
//! use std::fs::File;
//!
//! let file = File::open("/dev/input/event0").unwrap();
//! let mut d = Device::new_from_file(file).unwrap();
//!
//! loop {
//!     let a = d.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);
//!     match a {
//!         Ok(k) => println!("Event: time {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
//!                           k.1.time.tv_sec,
//!                           k.1.time.tv_usec,
//!                           k.1.event_type),
//!         Err(e) => (),
//!     }
//! }
//! ```
//!
//! ## Serialization
//! to use serialization, you muse enable the `serde` feature.
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! evdev-rs = { version = "0.4.0", features = ["serde"] }
//! ```

#[macro_use]
mod macros;
pub mod device;
pub mod enums;
pub mod logging;
pub mod uinput;
pub mod util;

use bitflags::bitflags;
use libc::{c_long, c_uint, c_void, suseconds_t, time_t};
use std::any::Any;
use std::convert::{TryFrom, TryInto};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use std::{io, ptr};

use enums::*;
use util::*;

use evdev_sys as raw;

#[doc(inline)]
pub use device::Device;
#[doc(inline)]
pub use device::UninitDevice;
#[doc(inline)]
pub use uinput::UInputDevice;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub enum GrabMode {
    /// Grab the device if not currently grabbed
    Grab = raw::LIBEVDEV_GRAB as isize,
    /// Ungrab the device if currently grabbed
    Ungrab = raw::LIBEVDEV_UNGRAB as isize,
}

bitflags! {
    pub struct ReadFlag: u32 {
        /// Process data in sync mode
        const SYNC = 1;
        /// Process data in normal mode
        const NORMAL = 2;
        /// Pretend the next event is a SYN_DROPPED and require the
        /// caller to sync
        const FORCE_SYNC = 4;
        /// The fd is not in O_NONBLOCK and a read may block
        const BLOCKING = 8;
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

#[cfg_attr(feature = "serde", derive(Serialize), derive(Deserialize))]
#[derive(Copy, Clone, Eq, Hash, PartialOrd, Ord, Debug, PartialEq)]
pub struct TimeVal {
    pub tv_sec: c_long,
    pub tv_usec: c_long,
}

impl TryFrom<SystemTime> for TimeVal {
    type Error = SystemTimeError;
    fn try_from(system_time: SystemTime) -> Result<Self, Self::Error> {
        let d = system_time.duration_since(UNIX_EPOCH)?;
        Ok(TimeVal {
            tv_sec: d.as_secs() as time_t,
            tv_usec: d.subsec_micros() as suseconds_t,
        })
    }
}

impl TryInto<SystemTime> for TimeVal {
    type Error = ();
    /// Fails if TimeVal.tv_usec is >= 10^6 or if the TimeVal is outside
    /// the range of SystemTime
    fn try_into(self) -> Result<SystemTime, Self::Error> {
        let secs = self.tv_sec.try_into().map_err(|_| ())?;
        let nanos = (self.tv_usec * 1000).try_into().map_err(|_| ())?;
        let duration = Duration::new(secs, nanos);
        UNIX_EPOCH.checked_add(duration).ok_or(())
    }
}

impl TimeVal {
    pub fn new(tv_sec: c_long, tv_usec: c_long) -> TimeVal {
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
            tv_usec: self.tv_usec,
        }
    }
}

/// The event structure itself
#[cfg_attr(feature = "serde", derive(Serialize), derive(Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputEvent {
    /// The time at which event occured
    pub time: TimeVal,
    pub event_type: EventType,
    pub event_code: EventCode,
    pub value: i32,
}

impl InputEvent {
    pub fn new(timeval: &TimeVal, code: &EventCode, value: i32) -> InputEvent {
        let (ev_type, _) = event_code_to_int(code);
        InputEvent {
            time: *timeval,
            event_type: int_to_event_type(ev_type).unwrap(),
            event_code: *code,
            value,
        }
    }

    pub fn from_raw(event: &libc::input_event) -> InputEvent {
        let ev_type = event.type_ as u32;
        let event_type = int_to_event_type(ev_type).unwrap();
        let event_code = int_to_event_code(ev_type, event.code as u32);
        InputEvent {
            time: TimeVal::from_raw(&event.time),
            event_type,
            event_code,
            value: event.value,
        }
    }

    pub fn as_raw(&self) -> libc::input_event {
        let (ev_type, ev_code) = event_code_to_int(&self.event_code);
        libc::input_event {
            time: self.time.as_raw(),
            type_: ev_type as u16,
            code: ev_code as u16,
            value: self.value,
        }
    }

    pub fn is_type(&self, ev_type: &EventType) -> bool {
        unsafe { raw::libevdev_event_is_type(&self.as_raw(), *ev_type as c_uint) == 1 }
    }

    pub fn is_code(&self, code: &EventCode) -> bool {
        let (ev_type, ev_code) = event_code_to_int(code);

        unsafe { raw::libevdev_event_is_code(&self.as_raw(), ev_type, ev_code) == 1 }
    }
}

/// Abstraction over structs which contain an inner `*mut libevdev`
pub trait LibevdevWrapper {
    fn raw(&self) -> *mut raw::libevdev;

    /// Forcibly enable an EventType/InputProp on this device, even if the underlying
    /// device does not support it. While this cannot make the device actually
    /// report such events, it will now return true for has().
    ///
    /// This is a local modification only affecting only this representation of
    /// this device.
    fn enable(&self, blob: &dyn Any) -> io::Result<()> {
        if let Some(ev_type) = blob.downcast_ref::<EventType>() {
            self.enable_event_type(ev_type)
        } else if let Some(ev_code) = blob.downcast_ref::<EventCode>() {
            self.enable_event_code(ev_code, None)
        } else if let Some(prop) = blob.downcast_ref::<InputProp>() {
            self.enable_property(prop)
        } else {
            Err(io::Error::from_raw_os_error(-1))
        }
    }

    /// Enables this property, a call to `set_file` will overwrite any previously set values
    ///
    /// Note: Please use the `enable` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn enable_property(&self, prop: &InputProp) -> io::Result<()> {
        let result =
            unsafe { raw::libevdev_enable_property(self.raw(), *prop as c_uint) as i32 };

        match result {
            0 => Ok(()),
            error => Err(io::Error::from_raw_os_error(-error)),
        }
    }

    /// Forcibly enable an event type on this device, even if the underlying
    /// device does not support it. While this cannot make the device actually
    /// report such events, it will now return true for libevdev_has_event_type().
    ///
    /// This is a local modification only affecting only this representation of
    /// this device.
    ///
    /// Note: Please use the `enable` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn enable_event_type(&self, ev_type: &EventType) -> io::Result<()> {
        let result =
            unsafe { raw::libevdev_enable_event_type(self.raw(), *ev_type as c_uint) };

        match result {
            0 => Ok(()),
            error => Err(io::Error::from_raw_os_error(-error)),
        }
    }

    /// Forcibly enable an event type on this device, even if the underlying
    /// device does not support it. While this cannot make the device actually
    /// report such events, it will now return true for libevdev_has_event_code().
    ///
    /// The last argument depends on the type and code:
    /// If type is EV_ABS, data must be a pointer to a struct input_absinfo
    /// containing the data for this axis.
    /// If type is EV_REP, data must be a pointer to a int containing the data
    /// for this axis.
    /// For all other types, the argument must be NULL.
    ///
    /// Note: Please use the `enable` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn enable_event_code(
        &self,
        ev_code: &EventCode,
        blob: Option<&dyn Any>,
    ) -> io::Result<()> {
        let (ev_type, ev_code) = event_code_to_int(ev_code);

        let data = blob
            .map(|data| {
                data.downcast_ref::<AbsInfo>()
                    .map(|absinfo| &absinfo.as_raw() as *const _ as *const c_void)
                    .unwrap_or_else(|| data as *const _ as *const c_void)
            })
            .unwrap_or_else(|| ptr::null() as *const _ as *const c_void);

        let result = unsafe {
            raw::libevdev_enable_event_code(
                self.raw(),
                ev_type as c_uint,
                ev_code as c_uint,
                data,
            )
        };

        match result {
            0 => Ok(()),
            error => Err(io::Error::from_raw_os_error(-error)),
        }
    }

    /// Forcibly disable an EventType/EventCode on this device, even if the
    /// underlying device provides it. This effectively mutes the respective set of
    /// events. has() will return false for this EventType/EventCode
    ///
    /// In most cases, a caller likely only wants to disable a single code, not
    /// the whole type.
    ///
    /// Disabling EV_SYN will not work. In Peter's Words "Don't shoot yourself
    /// in the foot. It hurts".
    ///
    /// This is a local modification only affecting only this representation of
    /// this device.
    fn disable(&self, blob: &dyn Any) -> io::Result<()> {
        if let Some(ev_type) = blob.downcast_ref::<EventType>() {
            self.disable_event_type(ev_type)
        } else if let Some(ev_code) = blob.downcast_ref::<EventCode>() {
            self.disable_event_code(ev_code)
        } else {
            Err(io::Error::from_raw_os_error(-1))
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
    ///
    /// Note: Please use the `disable` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn disable_event_type(&self, ev_type: &EventType) -> io::Result<()> {
        let result =
            unsafe { raw::libevdev_disable_event_type(self.raw(), *ev_type as c_uint) };

        match result {
            0 => Ok(()),
            error => Err(io::Error::from_raw_os_error(-error)),
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
    ///
    /// Note: Please use the `disable` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn disable_event_code(&self, code: &EventCode) -> io::Result<()> {
        let (ev_type, ev_code) = event_code_to_int(code);
        let result =
            unsafe { raw::libevdev_disable_event_code(self.raw(), ev_type, ev_code) };

        match result {
            0 => Ok(()),
            error => Err(io::Error::from_raw_os_error(-error)),
        }
    }

    /// Returns `true` if device support the InputProp/EventType/EventCode and false otherwise
    fn has(&self, blob: &dyn Any) -> bool {
        if let Some(ev_type) = blob.downcast_ref::<EventType>() {
            self.has_event_type(ev_type)
        } else if let Some(ev_code) = blob.downcast_ref::<EventCode>() {
            self.has_event_code(ev_code)
        } else if let Some(prop) = blob.downcast_ref::<InputProp>() {
            self.has_property(prop)
        } else {
            false
        }
    }

    /// Returns `true` if device support the property and false otherwise
    ///
    /// Note: Please use the `has` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn has_property(&self, prop: &InputProp) -> bool {
        unsafe { raw::libevdev_has_property(self.raw(), *prop as c_uint) != 0 }
    }

    /// Returns `true` is the device support this event type and `false` otherwise
    ///
    /// Note: Please use the `has` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn has_event_type(&self, ev_type: &EventType) -> bool {
        unsafe { raw::libevdev_has_event_type(self.raw(), *ev_type as c_uint) != 0 }
    }

    /// Return `true` is the device support this event type and code and `false` otherwise
    ///
    /// Note: Please use the `has` function instead. This function is only
    /// available for the sake of maintaining compatibility with libevdev.
    fn has_event_code(&self, code: &EventCode) -> bool {
        unsafe {
            let (ev_type, ev_code) = event_code_to_int(code);
            raw::libevdev_has_event_code(self.raw(), ev_type, ev_code) != 0
        }
    }
}

impl LibevdevWrapper for UninitDevice {
    fn raw(&self) -> *mut raw::libevdev {
        self.raw
    }
}

impl LibevdevWrapper for Device {
    fn raw(&self) -> *mut raw::libevdev {
        self.raw
    }
}
