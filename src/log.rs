use raw;

pub enum LogPriority {
    Error = raw::LIBEVDEV_LOG_ERROR as isize,
    Info = raw::LIBEVDEV_LOG_INFO as isize,
    Debug = raw::LIBEVDEV_LOG_DEBUG as isize,
}


pub fn set_log_priority(priority: LogPriority) {
    unsafe {
        raw::libevdev_set_log_priority(priority as i32);
    }
}

pub fn get_log_priority() -> LogPriority {
    unsafe {
        let priority = raw::libevdev_get_log_priority();
        match priority {
            raw::LIBEVDEV_LOG_ERROR => LogPriority::Error,
            raw::LIBEVDEV_LOG_INFO => LogPriority::Info,
            raw::LIBEVDEV_LOG_DEBUG => LogPriority::Debug,
            c => panic!("unknown log priority: {}", c),
        }
    }
}
