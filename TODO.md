## These function need to implemented in evdev-rs

* `int libevdev_enable_event_code(struct libevdev *dev, unsigned int type, unsigned int code, const void *data);`
* `int libevdev_kernel_set_led_values(struct libevdev *dev, ...);`

## We need to define this functions types and the corresponding functions

* libevdev_log_func_t
    * `void libevdev_set_log_function(libevdev_log_func_t logfunc, void *data);`
* libevdev_device_log_func_t
    * `void libevdev_set_device_log_function(struct libevdev *dev,
				      libevdev_device_log_func_t logfunc,
				      enum libevdev_log_priority priority,
				      void *data);`

## Add Documentation
