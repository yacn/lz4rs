#![allow(unstable)]
#![allow(non_snake_case)]

extern crate libc;

use libc::{c_uint, c_int, size_t, c_char};

use std::ffi::CString;

pub type LZ4F_errorCode_t = size_t;

#[repr(C)]
pub enum blockSizeID_t {
	LZ4F_default = 0,
	max64KB = 4,
	max256KB = 5,
	max1MB = 6,
	max4MB = 7,
}

#[repr(C)]
pub enum blockMode_t {
	blockLinked = 0,
	blockIndependent,
}

#[repr(C)]
pub enum contentChecksum_t {
	noContentChecksum = 0,
	contentChecksumEnabled,
}

#[repr(C)]
pub enum frameType_t {
	LZ4F_frame = 0,
	skippableFrame,
}

#[repr(C)]
pub struct LZ4F_frameInfo_t {
	pub blockSizeID: blockSizeID_t,
	pub blockMode: blockMode_t,
	pub contentChecksumFlag: contentChecksum_t,
	pub frameType: frameType_t,
	pub contentSize: u64,
	pub reserved: [c_uint; 2],
}

#[repr(C)]
pub struct LZ4F_preferences_t {
	pub frameInfo: LZ4F_frameInfo_t,
	pub compressionLevel: c_uint,
	pub autoFlush: c_uint,
	pub reserved: [c_uint; 4],
}

extern {

	// lz4frame.h
	//unsigned    LZ4F_isError(LZ4F_errorCode_t code);
	pub fn LZ4F_isError(code: size_t) -> c_uint;

	//const char* LZ4F_getErrorName(LZ4F_errorCode_t code);
	/* return error code string; useful for debugging */
	pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;

	//size_t LZ4F_compressFrameBound(size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
	pub fn LZ4F_compressFrameBound(srcSize: size_t, prefsPtr: *const LZ4F_preferences_t) -> size_t;

	//size_t LZ4F_compressFrame(void* dstBuffer, size_t dstMaxSize, const void* srcBuffer, size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
	pub fn LZ4F_compressFrame(dstBuffer: *mut u8, dstMaxSize: size_t, srcBuffer: *const u8, srcSize: size_t, prefsPtr: *const LZ4F_preferences_t) -> size_t;

}