#![allow(bad_style)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

extern crate libc;

use libc::{c_int, c_uint, c_char, int32_t};

#[repr(C)]
pub struct libevdev;

#[repr(C)]
pub struct input_absinfo {
    pub value: int32_t,
    pub minimum: int32_t,
    pub maximum: int32_t,
    pub fuzz: int32_t,
    pub flat: int32_t,
    pub resolution: int32_t,
}

extern {
    pub fn libevdev_new() -> *mut libevdev;
    pub fn libevdev_free(ctx: *mut libevdev);
    pub fn libevdev_set_fd(ctx: *mut libevdev, fd: c_int) -> c_int;
    pub fn libevdev_change_fd(ctx: *mut libevdev, fd: c_int) -> c_int;
    pub fn libevdev_get_fd(ctx: *mut libevdev) -> c_int;
    pub fn libevdev_grab(ctx: *mut libevdev, grab: c_int) -> c_int;
    pub fn libevdev_get_name(ctx: *const libevdev) -> *const c_char;
    pub fn libevdev_get_uniq(ctx: *const libevdev) -> *const c_char;
    pub fn libevdev_get_phys(ctx: *const libevdev) -> *const c_char;
    pub fn libevdev_get_abs_info(ctx: *const libevdev, code: c_uint) -> *const input_absinfo;
    pub fn libevdev_has_property(ctx: *const libevdev, prop: c_uint) -> c_int;
    pub fn libevdev_has_event_type(ctx: *const libevdev, type_: c_uint) -> c_int;
    pub fn libevdev_has_event_code(ctx: *const libevdev, type_: c_uint, code: c_uint) -> c_int;
    pub fn libevdev_fetch_event_value(ctx: *const libevdev,
                                  type_: c_uint,
                                  code: c_uint,
                                  value: *mut c_int) -> c_int;
    pub fn libevdev_event_type_get_name(type_: c_uint) -> *const c_char;
    pub fn libevdev_event_code_get_name(type_: c_uint, code: c_uint) -> *const c_char;
    pub fn libevdev_property_get_name(prop: c_uint) -> *const c_char;
}
