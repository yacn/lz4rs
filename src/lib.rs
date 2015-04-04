#![allow(unstable)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![feature(unsafe_destructor)]
#![allow(dead_code)]

extern crate libc;
extern crate collections;


mod liblz4;

pub mod frame;

pub fn version() -> usize {
	let result = unsafe { liblz4::core::LZ4_versionNumber() };
	return result as usize;
}
