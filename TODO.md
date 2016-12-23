## These function need to implemented in evdev-rs

* `int libevdev_new_from_fd(int fd, struct libevdev **dev);`
* `void libevdev_set_log_priority(enum libevdev_log_priority priority);`
* `enum libevdev_log_priority libevdev_get_log_priority(void);`
* `int libevdev_next_event(struct libevdev *dev, unsigned int flags, struct input_event *ev);`
* `void libevdev_set_name(struct libevdev *dev, const char *name);`
* `void libevdev_set_phys(struct libevdev *dev, const char *phys);`
* `void libevdev_set_uniq(struct libevdev *dev, const char *uniq);`
* `int libevdev_get_id_product(const struct libevdev *dev);`
* `void libevdev_set_id_product(struct libevdev *dev, int product_id);`
* `int libevdev_get_id_vendor(const struct libevdev *dev);`
* `void libevdev_set_id_vendor(struct libevdev *dev, int vendor_id);`
* `int libevdev_get_id_bustype(const struct libevdev *dev);`
* `void libevdev_set_id_bustype(struct libevdev *dev, int bustype);`
* `int libevdev_get_id_version(const struct libevdev *dev);`
* `void libevdev_set_id_version(struct libevdev *dev, int version);`
* `int libevdev_get_driver_version(const struct libevdev *dev);`
* `int libevdev_enable_property(struct libevdev *dev, unsigned int prop);`
* `int libevdev_get_abs_minimum(const struct libevdev *dev, unsigned int code);`
* `int libevdev_get_abs_maximum(const struct libevdev *dev, unsigned int code);`
* `int libevdev_get_abs_fuzz(const struct libevdev *dev, unsigned int code);`
* `int libevdev_get_abs_flat(const struct libevdev *dev, unsigned int code);`
* `int libevdev_get_abs_resolution(const struct libevdev *dev, unsigned int code);`
* `int libevdev_get_event_value(const struct libevdev *dev, unsigned int type, unsigned int code);`
* `int libevdev_set_event_value(struct libevdev *dev, unsigned int type, unsigned int code, int value);`
* `int libevdev_get_slot_value(const struct libevdev *dev, unsigned int slot, unsigned int code);`
* `int libevdev_set_slot_value(struct libevdev *dev, unsigned int slot, unsigned int code, int value);`
* `int libevdev_fetch_slot_value(const struct libevdev *dev, unsigned int slot, unsigned int code, int *value);`
* `int libevdev_get_num_slots(const struct libevdev *dev);`
* `int libevdev_get_current_slot(const struct libevdev *dev);`
* `void libevdev_set_abs_minimum(struct libevdev *dev, unsigned int code, int min);`
* `void libevdev_set_abs_maximum(struct libevdev *dev, unsigned int code, int max);`
* `void libevdev_set_abs_fuzz(struct libevdev *dev, unsigned int code, int fuzz);`
* `void libevdev_set_abs_flat(struct libevdev *dev, unsigned int code, int flat);`
* `void libevdev_set_abs_resolution(struct libevdev *dev, unsigned int code, int resolution);`
* `void libevdev_set_abs_info(struct libevdev *dev, unsigned int code, const struct input_absinfo *abs);`
* `int libevdev_enable_event_type(struct libevdev *dev, unsigned int type);`
* `int libevdev_disable_event_type(struct libevdev *dev, unsigned int type);`
* `int libevdev_enable_event_code(struct libevdev *dev, unsigned int type, unsigned int code, const void *data);`
* `int libevdev_disable_event_code(struct libevdev *dev, unsigned int type, unsigned int code);`
* `int libevdev_kernel_set_abs_info(struct libevdev *dev, unsigned int code, const struct input_absinfo *abs);`
* `int libevdev_kernel_set_led_value(struct libevdev *dev, unsigned int code, enum libevdev_led_value value);`
* `int libevdev_kernel_set_led_values(struct libevdev *dev, ...);`
* `int libevdev_set_clock_id(struct libevdev *dev, int clockid);`
* `int libevdev_event_is_type(const struct input_event *ev, unsigned int type);`
* `int libevdev_event_is_code(const struct input_event *ev, unsigned int type, unsigned int code);`
* `const char * libevdev_event_type_get_name(unsigned int type);`
* `const char * libevdev_event_code_get_name(unsigned int type, unsigned int code);`
* `const char* libevdev_property_get_name(unsigned int prop);`
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
