#![allow(unstable)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

extern crate libc;

use libc::{c_uint, c_int, size_t, c_char};


extern {
    // lz4.h
    // int LZ4_versionNumber(void)
    pub fn LZ4_versionNumber() -> c_int;
}