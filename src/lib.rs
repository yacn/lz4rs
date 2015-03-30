extern crate libc;
extern crate collections;

use libc::{c_uint, c_int, size_t, c_char};

use std::ffi::CString;

use collections::slice;

use std::io;
use std::mem;
use std::str;
use std::io::fs::File;
use std::io::IoError;
use std::io::IoErrorKind;
use std::io::IoErrorKind::ResourceUnavailable;

pub fn version() -> usize {
	let result = unsafe { LZ4_versionNumber() };
	return result as usize;
}

type LZ4F_errorCode_t = size_t;

enum blockSizeID_t {
	LZ4F_default = 0,
	max64KB = 4,
	max256KB = 5,
	max1MB = 6,
	max4MB = 7,
}

enum blockMode_t {
	blockLinked = 0,
	blockIndependent,
}

enum contentChecksum_t {
	noContentChecksum = 0,
	contentChecksumEnabled,
}

enum frameType_t {
	LZ4F_frame = 0,
	skippableFrame,
}

struct LZ4F_frameInfo_t {
	blockSizeID: blockSizeID_t,
	blockMode: blockMode_t,
	contentChecksumFlag: contentChecksum_t,
	frameType: frameType_t,
	// unsigned long long frameOSize /* size of uncompress (og) content, 0 => unknown */
	reserved: [c_uint; 5],
}

#[repr(C)]
struct LZ4F_preferences_t {
	frameInfo: LZ4F_frameInfo_t,
	compressionLevel: c_uint,
	autoFlush: c_uint,
	reserved: [c_uint; 4],
}

//typedef size_t LZ4F_errorCode_t;
extern {

	// lz4.h
	// int LZ4_versionNumber(void)
	fn LZ4_versionNumber() -> c_int;

	// lz4frame.h
	//unsigned    LZ4F_isError(LZ4F_errorCode_t code);
	fn LZ4F_isError(code: size_t) -> c_uint;

	//const char* LZ4F_getErrorName(LZ4F_errorCode_t code);
	/* return error code string; useful for debugging */
	fn LZ4F_getErrorName(code: size_t) -> *const c_char;

	//size_t LZ4F_compressFrameBound(size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
	fn LZ4F_compressFrameBound(srcSize: size_t, preferencesPtr: *const LZ4F_preferences_t) -> size_t;

	//size_t LZ4F_compressFrame(void* dstBuffer, size_t dstMaxSize, const void* srcBuffer, size_t srcSize, const LZ4F_preferences_t* preferencesPtr);
	fn LZ4F_compressFrame(dstBuffer: *mut u8, dstMaxSize: size_t, srcBuffer: *const u8, srcSize: size_t, preferencesPtr: *const LZ4F_preferences_t) -> size_t;

}

pub fn compress(src: &Path, dst: &Path) -> Result<(), IoError> {
	unsafe {

		println!("Compressing {:?} -> {:?}", src, dst);

		//let buf_size: usize = 32 * 1024;

		//let mut buf: [u8; buf_size] = [0; buf_size];

		let mut src_file = try!(File::open(src));
		let mut dst_file = try!(File::create(dst));

		let fstat = try!(src_file.stat());
		println!("got stat");

		let src_buf_size = fstat.size as usize;
		println!("src_buf_size: {:?}", src_buf_size);

		//let mut src_buf: [u8; src_buf_size] = [0; src_buf_size];
		let mut src_buf: Vec<u8> = Vec::with_capacity(src_buf_size);
		let mut src_buf_slice = src_buf.as_mut_slice();

		//let read_len = try!(src_file.read(src_buf.as_mut_slice()));
		//let read_len = try!(src_file.read(src_buf.as_mut_slice()));
		match src_file.read(src_buf_slice) {
			Ok(l) => println!("read {}", l),
			Err(ref e) if e.kind == IoErrorKind::EndOfFile => {},
			Err(e) => return Err(e),
		}
		println!("read src");
		println!("src_buf ({:?}):\n{:?}", src_buf_slice.len(), src_buf_slice);

		let preferences = LZ4F_preferences_t {
			frameInfo: LZ4F_frameInfo_t {
				blockSizeID: blockSizeID_t::LZ4F_default,
				blockMode: blockMode_t::blockLinked,
				contentChecksumFlag: contentChecksum_t::contentChecksumEnabled,
				frameType: frameType_t::LZ4F_frame,
				reserved: [0; 5],
			},
			compressionLevel: 0,
			autoFlush: 0,
			reserved: [0; 4],
		};

		//let maybe_err = unsafe { LZ4F_compressFrameBound(src_buf_size as size_t, &preferences) };
		//let dst_max_size = try!(check_error(maybe_err));
		let dst_max_size = try!(compress_frame_bound(src_buf_size, &preferences));
		println!("got max size: {:?}", dst_max_size);
		let mut dst_buf = Vec::with_capacity(dst_max_size);

		let maybe_err =
			LZ4F_compressFrame(dst_buf.as_mut_ptr(), dst_max_size as size_t, src_buf_slice.as_ptr(), src_buf_size as size_t, &preferences);

		let compressed_len = try!(check_error(maybe_err));
		println!("compressed frame: {:?}", compressed_len);
		println!("dst_buf: {:?}", dst_buf);

		let r = try!(dst_file.write(dst_buf.as_slice()));
		println!("wrote dst: {:?}", r);
		Ok(r)
	}

	/*let len = try!(file_in.read(&mut buf));

	if len == 0 { return Err("read zero err".to_string()); }

	while buf.len() > 0 {

	}*/

}

fn compress_frame_bound(src_size: usize, preferences: &LZ4F_preferences_t) -> Result<usize, IoError> {
	let maybe_err = unsafe { LZ4F_compressFrameBound(src_size as size_t, preferences) };
	check_error(maybe_err)
}

fn check_error(code: LZ4F_errorCode_t) -> Result<usize, IoError> {
	println!("checking: {:?}", code);
	unsafe {
		if LZ4F_isError(code) != 0 {
			let error_name: *const c_char = LZ4F_getErrorName(code);
			let err_name_len = libc::strlen(error_name);
			//let slice = mem::transmute(slice::from_raw_parts(error_name, err_name_len as usize + 1));
			let slice = mem::transmute(slice::from_raw_buf(&error_name, err_name_len as usize + 1));
			//let err_c_str_bytes = CString::from_ptr(error_name).to_bytes();
			//let err_str = str::from_utf8(err_c_str_bytes).unwrap().to_string();
			let err_str = str::from_utf8(slice).unwrap().to_string();
			let e = IoError {
				kind: IoErrorKind::OtherIoError,
				desc: "LZ4 error",
				detail: Some(err_str),
			};
			return Err(e);
		}
	}
	Ok(code as usize)
}


/*fn copy<R: Reader, W: Writer>(src: &mut R, dst: &mut W) -> Result<()> {
	let buf_size: usize = 32 * 1024;

	if len == 0 { break; }
	//try!(dst.write_all(&buf[0..len]));
	while buf.len() > 0 {
		match dst.write(buf) {
			Ok(0) => return Err("write zero err".to_string()),
			//Ok(n) => buf = &buf[n..],
			Ok(n) => buf = buf.slice_from(n),
			Err(ref e) if e.kind() == ResourceUnavailable => {},
			Err(e) => return Err(e),

		}
	}
	Ok(())
}*/






























