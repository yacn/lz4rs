//! This library contains bindings to the various LZ4 formats. Currently, only the `frame`
//! module is complete.

#![allow(unstable)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

extern crate libc;
extern crate collections;

pub use self::core::version;

mod liblz4;

pub mod frame;
pub mod core;
