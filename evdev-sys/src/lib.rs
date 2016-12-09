#![allow(bad_style)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

extern crate libc;

use libc::{c_int, c_uint, c_char, c_void, c_long};

pub type __enum_ty = libc::c_int;
pub type libevdev_read_flag = __enum_ty;
pub type libevdev_log_priority = __enum_ty;
pub type libevdev_grab_mode = __enum_ty;
pub type libevdev_read_status = __enum_ty;
pub type libevdev_led_value = __enum_ty;

pub const LIBEVDEV_READ_FLAG_SYNC: libevdev_read_flag = 1;
pub const LIBEVDEV_READ_FLAG_NORMAL: libevdev_read_flag = 2;
pub const LIBEVDEV_READ_FLAG_FORCE_SYNC: libevdev_read_flag = 4;
pub const LIBEVDEV_READ_FLAG_BLOCKING: libevdev_read_flag = 8;

pub const LIBEVDEV_LOG_ERROR: libevdev_log_priority = 10;
pub const LIBEVDEV_LOG_INFO: libevdev_log_priority = 20;
pub const LIBEVDEV_LOG_DEBUG: libevdev_log_priority = 30;

pub const LIBEVDEV_GRAB: libevdev_grab_mode = 3;
pub const LIBEVDEV_UNGRAB: libevdev_grab_mode = 4;

pub const LIBEVDEV_READ_STATUS_SUCCESS: libevdev_read_status = 0;
pub const LIBEVDEV_READ_STATUS_SYNC: libevdev_read_status = 1;

pub const LIBEVDEV_LED_ON: libevdev_led_value = 3;
pub const LIBEVDEV_LED_OFF: libevdev_led_value = 4;

#[repr(C)]
pub struct libevdev;

#[repr(C)]
pub struct input_absinfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}

#[repr(C)]
pub struct va_list {
    // TODO
}

#[repr(C)]
struct timeval {
       tv_sec: c_long,
       tv_usec: c_long,
}

#[repr(C)]
pub struct input_event {
   time: timeval,
   type_t: u16,
   code: u16,
   value: i32,
}

type libevdev_log_func_t = extern fn(*const libevdev,
                                     *mut c_void,
                                     *const c_char, c_int,
                                     *const c_char,
                                     *const c_char, va_list);

type libevdev_device_log_func_t = extern fn(*const libevdev,
                                            c_int, *mut c_void,
                                            *const c_char, c_int,
                                            *const c_char,
                                            *const c_char, va_list);


extern {
    pub fn libevdev_new() -> *mut libevdev;
    pub fn libevdev_new_from_fd(fd: c_int, ctx: *mut *mut libevdev) -> c_int;
    pub fn libevdev_free(ctx: *mut libevdev);
    pub fn libevdev_set_fd(ctx: *mut libevdev, fd: c_int) -> c_int;
    pub fn libevdev_change_fd(ctx: *mut libevdev, fd: c_int) -> c_int;
    pub fn libevdev_get_fd(ctx: *mut libevdev) -> c_int;
    pub fn libevdev_grab(ctx: *mut libevdev, grab: libevdev_grab_mode) -> c_int;
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
    pub fn libevdev_set_log_priority(priority: libevdev_log_priority);
    pub fn libevdev_set_log_function(logfunc: libevdev_log_func_t, data: *mut c_void);
    pub fn libevdev_get_log_priority() -> libevdev_log_priority;
    pub fn libevdev_set_device_log_function(ctx: *mut libevdev,
                                            logfunc: libevdev_device_log_func_t,
                                            priority: libevdev_log_priority,
                                            data: *mut c_void);

}
