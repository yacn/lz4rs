#![allow(unstable)]
#![allow(non_snake_case)]

extern crate libc;
extern crate collections;

use libc::{c_uint, c_int, size_t, c_char, c_void};

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
	LZ4F_decompressionContext_t,
	LZ4F_VERSION,
};

use liblz4::frame::{
	LZ4F_isError,
	LZ4F_getErrorName,
	LZ4F_compressFrameBound,
	LZ4F_compressFrame,
	LZ4F_getFrameInfo,
	LZ4F_createDecompressionContext,
	LZ4F_freeDecompressionContext,
	LZ4F_decompress,

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
			contentSize: src_buf_size as u64,
			reserved: [0; 2],
		},
		compressionLevel: 0,
		autoFlush: 0,
		reserved: [0; 4],
	};

	//println!("{:?}", preferences.frameInfo);

	let dst_max_size = try!(compress_frame_bound(src_buf_size, &preferences));
	println!("got max size: {:?}", dst_max_size);
	let mut dst_buf: Vec<u8> = Vec::with_capacity(dst_max_size);

	unsafe {
		let dst_buf_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
		let src_buf_ptr: *const c_void = src_buf.as_ptr() as *const c_void;
		let maybe_err =
			LZ4F_compressFrame(dst_buf_ptr, dst_max_size as size_t, src_buf_ptr, src_buf_size as size_t, &preferences);

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



pub fn decompress(src: &Path, dst: &Path) -> Result<(), IoError> {

	println!("Decompressing {:?} -> {:?}", src, dst);

	let mut src_file = try!(File::open(src));
	let mut dst_file = try!(File::create(dst));

	let fstat = try!(src_file.stat());
	//println!("got stat");

	let src_buf_size: usize = fstat.size as usize;
	let mut src_buf_size2: size_t = src_buf_size.clone() as size_t;
	//println!("src_buf_size: {:?}", src_buf_size);

	let mut src_buf: Vec<u8> = try!(src_file.read_to_end());
	//println!("read src");
	println!("src_buf ({:?})", src_buf.len());
	println!("{:?}", src_buf.slice_to(5));


	let mut dctx: *mut c_void = ptr::null_mut();
	let maybe_err = unsafe { LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION) };
	try!(check_error(maybe_err));

	let mut frame_info: Box<LZ4F_frameInfo_t> = Box::new(LZ4F_frameInfo_t {
		blockSizeID: blockSizeID_t::LZ4F_default,
		blockMode: blockMode_t::blockLinked,
		contentChecksumFlag: contentChecksum_t::contentChecksumEnabled,
		frameType: frameType_t::LZ4F_frame,
		contentSize: 0 as u64,
		reserved: [0; 2],
	});

	unsafe {
		let src_buf_ptr: *const c_void = src_buf.as_ptr() as *const c_void;
		let size_hint: size_t = LZ4F_getFrameInfo(dctx, &mut *frame_info, src_buf_ptr, &mut src_buf_size2);
		println!("{}", size_hint);
	}

	println!("{:?}", frame_info);
	println!("{}", src_buf_size2);

	unsafe {
		let mut out_buf: Vec<u8> = Vec::with_capacity(1024);

		let mut src_buf_ptr: *const c_void = src_buf.as_ptr().offset(src_buf_size2 as isize) as *const c_void;
		let src_buf_end_ptr: *const c_void = src_buf.as_ptr().offset(src_buf_size as isize) as *const c_void;

		while src_buf_ptr < src_buf_end_ptr {
			let mut out_buf_size: size_t = 1024;
			let mut src_size_ptr: size_t = 1024;
			let mut out_buf_ptr: *mut c_void = out_buf.as_mut_ptr() as *mut c_void;
			let maybe_ec = LZ4F_decompress(dctx, out_buf_ptr, &mut out_buf_size, src_buf_ptr, &mut src_size_ptr, ptr::null());
			let rv = try!(check_error(maybe_ec));
			out_buf.set_len(out_buf_size as usize);
			dst_file.write(out_buf.as_slice());
			if rv == 0 { break; }
			out_buf.clear();
			src_buf_ptr = src_buf_ptr.offset(src_size_ptr as isize);
		}
		LZ4F_freeDecompressionContext(dctx);
	}


	Ok(())

}
