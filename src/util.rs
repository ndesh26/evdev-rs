use enums::*;
use libc::{c_char, c_uint};
use raw;
use std::fmt;
use std::ffi::{CStr, CString};

pub(crate) fn ptr_to_str(ptr: *const c_char) -> Option<&'static str> {
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

pub struct EventTypeIterator {
    current: EventType
}

pub struct EventCodeIterator {
    current: EventCode
}

pub struct InputPropIterator {
    current: InputProp
}

pub fn event_code_to_int(event_code: &EventCode) -> (c_uint, c_uint) {
    let mut ev_type: c_uint = 0;
    let mut ev_code: c_uint = 0;
    match event_code.clone() {
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
        EventCode::EV_UNK { event_type, event_code } => {
            ev_type = event_type as c_uint;
            ev_code = event_code as c_uint;
        },
        _ => {
            warn!("Event code not found");
        }
    }

    (ev_type, ev_code)

}

pub fn int_to_event_code(event_type: c_uint, event_code: c_uint) -> Option<EventCode> {
    let ev_type: EventType = int_to_event_type(event_type as u32).unwrap();

    let ev_code = match ev_type {
        EventType::EV_SYN =>    match int_to_ev_syn(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_SYN(k)),
                                },
        EventType::EV_KEY =>    match int_to_ev_key(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_KEY(k)),

                                },
        EventType::EV_ABS =>    match int_to_ev_abs(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_ABS(k)),
                                },
        EventType::EV_REL =>    match int_to_ev_rel(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_REL(k)),
                                },
        EventType::EV_MSC =>    match int_to_ev_msc(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_MSC(k)),
                                },
        EventType::EV_SW =>     match int_to_ev_sw(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_SW(k)),
                                },
        EventType::EV_LED =>    match int_to_ev_led(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_LED(k)),
                                },
        EventType::EV_SND =>    match int_to_ev_snd(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_SND(k)),
                                },
        EventType::EV_REP =>    match int_to_ev_rep(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_REP(k)),
                                },
        EventType::EV_FF =>     match int_to_ev_ff(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_FF(k)),
                                },
        EventType::EV_PWR =>    Some(EventCode::EV_PWR),
        EventType::EV_FF_STATUS => match int_to_ev_ff(event_code as u32) {
                                    None => None,
                                    Some(k) => Some(EventCode::EV_FF_STATUS(k)),
                                },
        EventType::EV_UNK =>    None,
        EventType::EV_MAX =>    Some(EventCode::EV_MAX),
    };

    match ev_code {
        Some(_) => ev_code,
        None => Some(EventCode::EV_UNK {
            event_type: event_type as u32,
            event_code: event_code as u32,
        }),
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ptr_to_str(unsafe {
            raw::libevdev_event_type_get_name(self.clone() as c_uint)
        }).unwrap_or(""))
    }
}

impl fmt::Display for EventCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ev_type, ev_code) = event_code_to_int(self);
        write!(f, "{}", ptr_to_str(unsafe {
            raw::libevdev_event_code_get_name(ev_type, ev_code)
        }).unwrap_or(""))
    }
}

impl fmt::Display for InputProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ptr_to_str(unsafe {
            raw::libevdev_property_get_name(self.clone() as c_uint)
        }).unwrap_or(""))
    }
}

impl EventType {
    pub fn iter(&self) -> EventTypeIterator {
        EventTypeIterator { current: self.clone() }
    }

    /// The given type constant for the passed name or Errno if not found.
    pub fn from_str(name: &str) -> Option<EventType> {
        let name = CString::new(name).unwrap();
        let result = unsafe {
            raw::libevdev_event_type_from_name(name.as_ptr())
        };

        match result {
            -1 => None,
             k => int_to_event_type(k as u32),
        }
    }

    /// The max value defined for the given event type, e.g. ABS_MAX for a type
    /// of EV_ABS, or Errno for an invalid type.
    pub fn get_max(ev_type: &EventType) -> Option<i32> {
        let result = unsafe {
            raw::libevdev_event_type_get_max(ev_type.clone() as c_uint)
        };

        match result {
            -1 => None,
             k => Some(k),
        }
    }
}

impl EventCode {
    pub fn iter(&self) -> EventCodeIterator {
        EventCodeIterator { current: self.clone() }
    }

    /// Look up an event code by its type and name. Event codes start with a fixed
    /// prefix followed by their name (eg., "ABS_X"). The prefix must be included in
    /// the name. It returns the constant assigned to the event code or Errno if not
    /// found.
    pub fn from_str(ev_type: &EventType, name: &str) -> Option<EventCode> {
        let name = CString::new(name).unwrap();
        let result = unsafe {
            raw::libevdev_event_code_from_name(ev_type.clone() as c_uint, name.as_ptr())
        };

        match result {
            -1 => None,
             k => int_to_event_code(ev_type.clone() as u32, k as u32),
        }
    }
}

impl InputProp {
    pub fn iter(&self) -> InputPropIterator {
        InputPropIterator { current: self.clone() }
    }

    /// Look up an input property by its name. Properties start with the fixed
    /// prefix "INPUT_PROP_" followed by their name (eg., "INPUT_PROP_POINTER").
    /// The prefix must be included in the name. It returns the constant assigned
    /// to the property or Errno if not found.
    pub fn from_str(name: &str) -> Option<InputProp> {
        let name = CString::new(name).unwrap();
        let result = unsafe {
            raw::libevdev_property_from_name(name.as_ptr())
        };

        match result {
            -1 => None,
             k => int_to_input_prop(k as u32),
        }
    }
}

// Iterator trait for the enum iterators
impl Iterator for EventTypeIterator {
    type Item = EventType;

    fn next(&mut self) -> Option<EventType> {
        match self.current {
            EventType::EV_MAX => {
                return None;
            }
            _ => {
                let mut raw_code = (self.current.clone() as u32) + 1;
                loop {
                    match int_to_event_type(raw_code) {
                        Some(x) => {
                            let code = self.current.clone();
                            self.current = x;
                            return Some(code);
                        }
                        None => raw_code += 1,
                    }
                }
            }
        }
    }
}

impl Iterator for EventCodeIterator {
    type Item = EventCode;

    fn next(&mut self) -> Option<EventCode> {
        match self.current.clone() {
            EventCode::EV_SYN(code) => {
                match code {
                    EV_SYN::SYN_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_KEY(EV_KEY::KEY_RESERVED);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_syn(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_SYN(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_KEY(code) => {
                match code {
                    EV_KEY::KEY_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_REL(EV_REL::REL_X);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_key(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_KEY(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_REL(code) => {
                match code {
                    EV_REL::REL_MAX=> {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_ABS(EV_ABS::ABS_X);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_rel(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_REL(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_ABS(code) => {
                match code {
                    EV_ABS::ABS_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_MSC(EV_MSC::MSC_SERIAL);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_abs(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_ABS(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_MSC(code) => {
                match code {
                    EV_MSC::MSC_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_SW(EV_SW::SW_LID);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_msc(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_MSC(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_SW(code) => {
                match code {
                    EV_SW::SW_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_LED(EV_LED::LED_NUML);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_sw(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_SW(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_LED(code) => {
                match code {
                    EV_LED::LED_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_SND(EV_SND::SND_CLICK);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_led(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_LED(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_SND(code) => {
                match code {
                    EV_SND::SND_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_REP(EV_REP::REP_DELAY);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_snd(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_SND(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_REP(code) => {
                match code {
                    EV_REP::REP_MAX => {
                        let ev_code = self.current.clone();
                        self.current = EventCode::EV_FF(EV_FF::FF_STATUS_STOPPED);
                        return Some(ev_code);
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_rep(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_REP(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            EventCode::EV_FF(code) => {
                match code {
                    EV_FF::FF_MAX => {
                        return None
                    }
                    _ => {
                        let mut raw_code = (code as u32) + 1;
                        loop {
                            match int_to_ev_ff(raw_code) {
                                Some(x) => {
                                    let ev_code = self.current.clone();
                                    self.current = EventCode::EV_FF(x);
                                    return Some(ev_code);
                                }
                                None => raw_code += 1,
                            }
                        }
                    }
                }
            }
            _ => None,
        }
    }
}

impl Iterator for InputPropIterator {
    type Item = InputProp;

    fn next(&mut self) -> Option<InputProp> {
        match self.current {
            InputProp::INPUT_PROP_MAX => {
                return None;
            }
            _ => {
                let mut raw_enum = (self.current.clone() as u32) + 1;
                loop {
                    match int_to_input_prop(raw_enum) {
                        Some(x) => {
                            let prop = self.current.clone();
                            self.current = x;
                            return Some(prop);
                        }
                        None => raw_enum += 1,
                    }
                }
            }
        }
    }
}

