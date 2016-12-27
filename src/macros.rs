macro_rules! string_getter {
    ( $( $func_name:ident, $c_func: ident ),* ) => {
        $(
            pub fn $func_name (&self) -> Option<&str> {
                ptr_to_str(unsafe {
                    raw::$c_func(self.raw)
                })
            }
        )*
    };
}

macro_rules! string_setter {
    ( $( $func_name:ident, $c_func: ident ),* ) => {
        $(
            pub fn $func_name (&self, field: &str) {
                let field = CString::new(field).unwrap();
                unsafe {
                    raw::$c_func(self.raw, field.as_ptr())
                }
            }
        )*
    };
}

macro_rules! product_getter {
    ( $( $func_name:ident, $c_func: ident ),* ) => {
        $(
            pub fn $func_name (&self) -> i32 {
                unsafe {
                    raw::$c_func(self.raw) as i32
                }
            }
        )*
    };
}

macro_rules! product_setter {
    ( $( $func_name:ident, $c_func: ident ),* ) => {
        $(
            pub fn $func_name (&self, field: i32) {
                unsafe {
                    raw::$c_func(self.raw, field as c_int);
                }
            }
        )*
    };
}

macro_rules! abs_getter {
    ( $( $func_name:ident, $c_func: ident ),* ) => {
        $(
            pub fn $func_name (&self,
                               code: u32) -> Result<i32, nix::errno::Errno> {
                let result = unsafe {
                    raw::$c_func(self.raw, code as c_uint) as i32
                };

                match result {
                    0 => Err(nix::errno::from_i32(0)),
                    k => Ok(k)
                }
            }
        )*
    };
}

macro_rules! abs_setter {
    ( $( $func_name:ident, $c_func: ident ),* ) => {
        $(
            pub fn $func_name (&self,
                               code: u32,
                               val: i32) {
                unsafe {
                    raw::$c_func(self.raw, code as c_uint, val as c_int);
                }
            }
        )*
    };
}
