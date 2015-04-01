#![allow(unstable)]
#![allow(non_snake_case)]

extern crate libc;
extern crate collections;

use libc::{c_uint, c_int, size_t, c_char};

use std::ffi::CString;

use collections::slice;

use std::io;
use std::mem;
use std::ptr;
use std::str;
use std::io::fs::File;
use std::io::IoError;
use std::io::IoErrorKind;
use std::io::IoErrorKind::ResourceUnavailable;

// structs/types
use liblz4::frame::{
	LZ4F_preferences_t,
	LZ4F_frameInfo_t,
	frameType_t,
	contentChecksum_t,
	blockMode_t,
	blockSizeID_t,
	LZ4F_errorCode_t,
};

use liblz4::frame::{
	LZ4F_isError,
	LZ4F_getErrorName,
	LZ4F_compressFrameBound,
	LZ4F_compressFrame,
};


mod liblz4;

pub fn version() -> usize {
	let result = unsafe { liblz4::core::LZ4_versionNumber() };
	return result as usize;
}


// use liblz4::frame::*

pub fn compress(src: &Path, dst: &Path) -> Result<(), IoError> {

	println!("Compressing {:?} -> {:?}", src, dst);

	let mut src_file = try!(File::open(src));
	let mut dst_file = try!(File::create(dst));

	let fstat = try!(src_file.stat());
	println!("got stat");

	let src_buf_size = fstat.size as usize;
	println!("src_buf_size: {:?}", src_buf_size);

	let mut src_buf: Vec<u8> = try!(src_file.read_to_end());
	println!("read src");
	println!("src_buf ({:?})", src_buf.len());
	println!("{:?}", src_buf.slice_to(5));


	let preferences = LZ4F_preferences_t {
		frameInfo: LZ4F_frameInfo_t {
			blockSizeID: blockSizeID_t::LZ4F_default,
			blockMode: blockMode_t::blockLinked,
			contentChecksumFlag: contentChecksum_t::contentChecksumEnabled,
			frameType: frameType_t::LZ4F_frame,
			contentSize: 0,
			reserved: [0; 2],
		},
		compressionLevel: 0,
		autoFlush: 0,
		reserved: [0; 4],
	};

	let dst_max_size = try!(compress_frame_bound(src_buf_size, &preferences));
	println!("got max size: {:?}", dst_max_size);
	let mut dst_buf: Vec<u8> = Vec::with_capacity(dst_max_size);

	unsafe {
		let maybe_err =
			LZ4F_compressFrame(dst_buf.as_mut_ptr(), dst_max_size as size_t, src_buf.as_ptr(), src_buf_size as size_t, ptr::null());

		let compressed_len = try!(check_error(maybe_err));
		println!("compressed frame: {:?}", compressed_len);
		dst_buf.set_len(compressed_len);
		println!("dst_buf: {:?}", dst_buf.slice_to(2));
	}
	Ok(try!(dst_file.write(dst_buf.as_slice())))

}

fn check_error(code: LZ4F_errorCode_t) -> Result<usize, IoError> {
	println!("checking: {:?}", code);
	if is_error(code) {
		let emsg = unsafe { get_error_string(code) };
		Err(IoError {
			kind: IoErrorKind::OtherIoError,
			desc: "LZ4",
			detail: Some(emsg),
		})
	} else {
		Ok(code as usize)
	}
}

fn compress_frame_bound(src_size: usize, preferences: &LZ4F_preferences_t) -> Result<usize, IoError> {
	let maybe_err = unsafe { LZ4F_compressFrameBound(src_size as size_t, preferences) };
	check_error(maybe_err)
}

fn is_error(code: LZ4F_errorCode_t) -> bool {
	let result = unsafe { LZ4F_isError(code) };
	result != 0
}

unsafe fn str_from_ptr(ptr: *const c_char) -> String {
	let len: usize = (libc::strlen(ptr) as usize) + 1;
	let char_slice: &[c_char] = slice::from_raw_buf(&ptr, len);
	let byte_slice: &[u8] = mem::transmute(char_slice);
	str::from_utf8(byte_slice).unwrap().to_string()
}

unsafe fn get_error_string(code: LZ4F_errorCode_t) -> String {
	let emsg_ptr: *const c_char = LZ4F_getErrorName(code);
	str_from_ptr(emsg_ptr)
}



