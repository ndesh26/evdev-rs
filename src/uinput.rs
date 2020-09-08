use crate::device::Device;
use crate::InputEvent;
use libc::c_int;
use std::io;
use std::os::unix::io::RawFd;

use crate::util::*;

use evdev_sys as raw;

/// Opaque struct representing an evdev uinput device
pub struct UInputDevice {
    raw: *mut raw::libevdev_uinput,
}

impl UInputDevice {
    /// Create a uinput device based on the given libevdev device.
    ///
    /// The uinput device will be an exact copy of the libevdev device, minus
    /// the bits that uinput doesn't allow to be set.
    pub fn create_from_device(device: &Device) -> io::Result<UInputDevice> {
        let mut libevdev_uinput = std::ptr::null_mut();
        let result = unsafe {
            raw::libevdev_uinput_create_from_device(
                device.raw,
                raw::LIBEVDEV_UINPUT_OPEN_MANAGED,
                &mut libevdev_uinput,
            )
        };

        match result {
            0 => Ok(UInputDevice {
                raw: libevdev_uinput,
            }),
            error => Err(io::Error::from_raw_os_error(-error)),
        }
    }

    string_getter!(
        #[doc = "Return the device node representing this uinput device.

This relies on `libevdev_uinput_get_syspath()` to provide a valid syspath."],
        devnode, libevdev_uinput_get_devnode
    );

    string_getter!(#[doc = "Return the syspath representing this uinput device.

If the UI_GET_SYSNAME ioctl not available, libevdev makes an educated
guess. The UI_GET_SYSNAME ioctl is available since Linux 3.15.

The syspath returned is the one of the input node itself
(e.g. /sys/devices/virtual/input/input123), not the syspath of the
device node returned with libevdev_uinput_get_devnode()."],
        syspath, libevdev_uinput_get_syspath);

    /// Return the file descriptor used to create this uinput device.
    ///
    /// This is an fd pointing to /dev/uinput. This file descriptor may be used
    /// to write events that are emitted by the uinput device.
    /// # Safety
    /// Closing this file descriptor (i.e. by creating a File from it, then 
    /// dropping the file) invalidates this UInputDevice
    pub unsafe fn as_fd(&self) -> Option<RawFd> {
        match raw::libevdev_uinput_get_fd(self.raw) {
            0 => None,
            result => Some(result)
        }
    }

    /// Return the file descriptor used to create this uinput device.
    ///
    /// This is an fd pointing to /dev/uinput. This file descriptor may be used
    /// to write events that are emitted by the uinput device.
    pub fn into_fd(self) -> Option<RawFd> {
        match unsafe { raw::libevdev_uinput_get_fd(self.raw) } {
            0 => None,
            result => Some(result)
        }
    }

    /// Post an event through the uinput device.
    ///
    /// It is the caller's responsibility that any event sequence is terminated
    /// with an EV_SYN/SYN_REPORT/0 event. Otherwise, listeners on the device
    /// node will not see the events until the next EV_SYN event is posted.
    pub fn write_event(&self, event: &InputEvent) -> io::Result<()> {
        let (ev_type, ev_code) = event_code_to_int(&event.event_code);
        let ev_value = event.value as c_int;

        let result = unsafe {
            raw::libevdev_uinput_write_event(self.raw, ev_type, ev_code, ev_value)
        };

        match result {
            0 => Ok(()),
            error => Err(io::Error::from_raw_os_error(-error)),
        }
    }
}

impl Drop for UInputDevice {
    fn drop(&mut self) {
        unsafe {
            raw::libevdev_uinput_destroy(self.raw);
        }
    }
}
