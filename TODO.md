## These function need to implemented in evdev-rs

* `int libevdev_next_event(struct libevdev *dev, unsigned int flags, struct input_event *ev);`
* `int libevdev_enable_event_code(struct libevdev *dev, unsigned int type, unsigned int code, const void *data);`
* `int libevdev_kernel_set_led_value(struct libevdev *dev, unsigned int code, enum libevdev_led_value value);`
* `int libevdev_kernel_set_led_values(struct libevdev *dev, ...);`
* `int libevdev_set_clock_id(struct libevdev *dev, int clockid);`
* `int libevdev_event_is_type(const struct input_event *ev, unsigned int type);`
* `int libevdev_event_is_code(const struct input_event *ev, unsigned int type, unsigned int code);`
* `int libevdev_event_type_get_max(unsigned int type);`
* `int libevdev_event_type_from_name(const char *name);`
* `int libevdev_event_type_from_name_n(const char *name, size_t len);`
* `int libevdev_event_code_from_name(unsigned int type, const char *name);`
* `int libevdev_event_code_from_name_n(unsigned int type, const char *name,`
* `int libevdev_property_from_name(const char *name);`
* `int libevdev_property_from_name_n(const char *name, size_t len);`
* `int libevdev_get_repeat(const struct libevdev *dev, int *delay, int *period);`

## We need to define this functions types and the corresponding functions

* libevdev_log_func_t
    * `void libevdev_set_log_function(libevdev_log_func_t logfunc, void *data);`
* libevdev_device_log_func_t
    * `void libevdev_set_device_log_function(struct libevdev *dev,
				      libevdev_device_log_func_t logfunc,
				      enum libevdev_log_priority priority,
				      void *data);`

## Add Documentation
