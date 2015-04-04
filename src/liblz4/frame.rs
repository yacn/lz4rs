//! Bindings to functions contained within lz4frame.h

#![allow(unstable)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

extern crate libc;

use libc::{c_uint, c_int, size_t, c_char, c_void};

use std::ffi::CString;

pub type LZ4F_errorCode_t = size_t;

#[derive(Show)]
#[repr(C)]
pub enum blockSizeID_t {
	LZ4F_default = 0,
	max64KB = 4,
	max256KB = 5,
	max1MB = 6,
	max4MB = 7,
}

#[derive(Show)]
#[repr(C)]
pub enum blockMode_t {
	blockLinked = 0,
	blockIndependent,
}

#[derive(Show)]
#[repr(C)]
pub enum contentChecksum_t {
	noContentChecksum = 0,
	contentChecksumEnabled,
}

#[derive(Show)]
#[repr(C)]
pub enum frameType_t {
	LZ4F_frame = 0,
	skippableFrame,
}

#[derive(Show)]
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

pub const LZ4F_VERSION: c_uint = 100;

pub type LZ4F_decompressionContext_t = *mut c_void;

#[repr(C)]
pub struct LZ4F_decompressOptions_t {
	stableDst: c_uint,
	reserved: [c_uint; 3],
}

// typedef struct {
//  unsigned stableSrc;    /* 1 == src content will remain available on future calls to LZ4F_compress(); avoid saving src content within tmp buffer as future dictionary */
//  unsigned reserved[3];
// } LZ4F_compressOptions_t;

// #define LZ4F_VERSION 100
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
	pub fn LZ4F_compressFrame(dstBuffer: *mut c_void, dstMaxSize: size_t, srcBuffer: *const c_void, srcSize: size_t, prefsPtr: *const LZ4F_preferences_t) -> size_t;

	//LZ4F_errorCode_t LZ4F_createCompressionContext(LZ4F_compressionContext_t* cctxPtr, unsigned version);
	fn LZ4F_createCompressionContext(cctxPtr: *mut c_void, version: c_uint) -> LZ4F_errorCode_t;

	// TODO
	//LZ4F_errorCode_t LZ4F_freeCompressionContext(LZ4F_compressionContext_t cctx);
	//fn LZ4F_freeCompressionContext(cctx: ...) -> LZ4F_errorCode_t

	//size_t LZ4F_compressBegin(LZ4F_compressionContext_t cctx, void* dstBuffer, size_t dstMaxSize, const LZ4F_preferences_t* prefsPtr);
	//fn LZ4F_compressBegin(cctx: ..., dstBuffer: *mut u8, dstMaxSize: size_t, prefsPtr: ...)

	//fn LZ4F_decompress(dctx: *mut c_void, dstBuffer: *mut c_void, 

	pub fn LZ4F_decompress(dctx: *mut c_void, dstBuffer: *mut c_void, dstSizePtr: *mut size_t, srcBuffer: *const c_void, srcSizePtr: *mut size_t, dOptPtr: *const LZ4F_decompressOptions_t) -> size_t;

	//size_t LZ4F_getFrameInfo(LZ4F_decompressionContext_t dctx,
                         //LZ4F_frameInfo_t* frameInfoPtr,
                         //const void* srcBuffer, size_t* srcSizePtr);

	pub fn LZ4F_getFrameInfo(dctx: *mut c_void, frameInfoPtr: *mut LZ4F_frameInfo_t, srcBuffer: *const c_void, srcSizePtr: *mut size_t) -> size_t;

	pub fn LZ4F_createDecompressionContext(dctxPtr: *mut *mut c_void, version: c_uint) -> LZ4F_errorCode_t;
	pub fn LZ4F_freeDecompressionContext(dctx: *mut c_void) -> LZ4F_errorCode_t;

}













