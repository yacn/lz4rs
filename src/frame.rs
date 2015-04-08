extern crate libc;
extern crate collections;

use libc::{size_t, c_char, c_void};

use collections::slice;

use std::mem;
use std::ptr;
use std::str;
use std::io::fs::File;
use std::io::{IoResult, IoError};
use std::io::IoErrorKind;
use std::io::IoErrorKind::{EndOfFile};

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

/// COMPRESS
/// Input: path to the source file and path to the distant file
/// Output: doesn't output anything but creates a compressed version of the file or throws an error
pub fn compress(src: &Path, dst: &Path) -> Result<(), IoError> {

  println!("Compressing {:?} -> {:?}", src, dst);

  let mut src_file = try!(File::open(src));
  let mut dst_file = try!(File::create(dst));

  let fstat = try!(src_file.stat());
  println!("got stat");

  let src_buf_size = fstat.size as usize;
  println!("src_buf_size: {:?}", src_buf_size);

  let mut src_buf: Vec<u8> = Vec::with_capacity(src_buf_size);

  unsafe {
    src_buf.set_len(src_buf_size);
  }

  try!(src_file.read(src_buf.as_mut_slice()));

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


  let dst_max_size = try!(compress_frame_bound(src_buf_size, &preferences));
  let mut dst_buf: Vec<u8> = Vec::with_capacity(dst_max_size);

  unsafe {
    let dst_buf_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let src_buf_ptr: *const c_void = src_buf.as_ptr() as *const c_void;
    let maybe_err =
      LZ4F_compressFrame(dst_buf_ptr, dst_max_size as size_t, src_buf_ptr, src_buf_size as size_t, &preferences);

    let compressed_len = try!(check_error(maybe_err));
    dst_buf.set_len(compressed_len);
  }
  Ok(try!(dst_file.write(dst_buf.as_slice())))
}
/// capture errors
/// Input: Error code
/// Output: usize if not an error, IoError otherwise
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
/// get the minimum value of dstMaxSize
/// Input: source size and preferences
/// Output: usize if got the minimum value, otherwise error
fn compress_frame_bound(src_size: usize, preferences: &LZ4F_preferences_t) -> Result<usize, IoError> {
  let maybe_err = unsafe { LZ4F_compressFrameBound(src_size as size_t, preferences) };
  check_error(maybe_err)
}
/// Provides the minimum size of Dst buffer given srcSize to handle worst case situations
/// Input: source size and preferences
/// Output: usize if got the minimum value, otherwise error
fn compress_bound(src_size: usize, preferences: Option<&LZ4F_preferences_t>) -> Result<usize, IoError> {

  let maybe_err = match preferences {
    Some(p) => unsafe { LZ4F_compressBound(src_size as size_t, p) },
    None => unsafe { LZ4F_compressBound(src_size as size_t, ptr::null()) },
  };
  check_error(maybe_err)
}
///checks if the the code is an error
/// Input: code
/// Output: bool
fn is_error(code: LZ4F_errorCode_t) -> bool {
  let result = unsafe { LZ4F_isError(code) };
  result != 0
}
///
unsafe fn str_from_ptr(ptr: *const c_char) -> String {
  let len: usize = (libc::strlen(ptr) as usize) + 1;
  let char_slice: &[c_char] = slice::from_raw_buf(&ptr, len);
  let byte_slice: &[u8] = mem::transmute(char_slice);
  str::from_utf8(byte_slice).unwrap().to_string()
}
///get error's name
/// Inpute: code
/// Output: error's name
unsafe fn get_error_string(code: LZ4F_errorCode_t) -> String {
  let emsg_ptr: *const c_char = LZ4F_getErrorName(code);
  str_from_ptr(emsg_ptr)
}

/*
struct Compressor<W> {
  inner: W,
  cctx: LZ4F_compressionContext_t,
  buffer: Vec<u8>,
  buf_offset: usize,
}


impl<W: Writer> Compressor<W> {
  fn new(src: W, src_size: Option<usize>) -> Result<Compressor<W>, IoError> {
    let srcsize: usize = src_size.unwrap_or(1024);
    let max_compressed_size: usize = try!(compress_bound(srcsize, None));
    let ctx: *mut c_void = ptr::null_mut();
    unsafe { LZ4F_createCompressionContext(&mut ctx, LZ4F_VERSION); }

    let buf: Vec<u8> = Vec::with_capacity(max_compressed_size);

    unsafe {
      LZ4F
    }

    Ok(Compressor {
      inner: src,
      cctx: ctx,
      buffer: buf,
      buf_offset: 0,
    })
  }
}
*/

/// DECOMPRESSOR struct

struct Decompressor<R> {
  inner: R,
  dctx: LZ4F_decompressionContext_t,
  buffer: Vec<u8>,
  buf_size: usize,
  eof: bool,
  buf_offset: usize,
}
/// functions for Decpmpressor struct
impl<R: Reader> Decompressor<R> {
  /// creates a new decompressor struct
  fn new(src: R, buf_size: Option<usize>) -> Decompressor<R> {
    let mut ctx: *mut c_void = ptr::null_mut();
    unsafe { LZ4F_createDecompressionContext(&mut ctx, LZ4F_VERSION); }

    let size: usize = buf_size.unwrap_or(1024);

    let mut buf: Vec<u8> = Vec::with_capacity(size);
    unsafe { buf.set_len(size); }

    Decompressor {
      inner: src,
      dctx: ctx,
      buffer: buf,
      buf_size: size,
      eof: false,
      buf_offset: size,
    }
  }
}

#[unsafe_destructor]
/// implement trait Drop for Decompressor with LZ4F_freeDecompressionContext
/// from lz4 library
impl<R: Reader> ::std::ops::Drop for Decompressor<R> {
  fn drop(&mut self) {
    unsafe {
      LZ4F_freeDecompressionContext(self.dctx);
    }
  }
}
/// implement trait Reader for Decompressor
impl<R: Reader> Reader for Decompressor<R> {
  /// read the decompressor
  /// if the function reach the end of file then it returns an EOF error
  /// otherwise, it returns current position
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {

    if self.eof {
      return Err(IoError {
        kind: IoErrorKind::EndOfFile,
        desc: "No more to decompress",
        detail: None,
      });
    }

    // offset indicating where we are currently in the provided buffer `buf'
    let mut dst_offset: usize = 0;

    // while our position in `buf' is < its length
    while dst_offset < buf.len() {

      // where we read up to `self.buffer.len()` compressed bytes into self.buffer
      // self.buffer.len == self.buf_size
      if self.buf_offset >= self.buf_size {
        // set our position in `self.buffer' to 0
        self.buf_offset = 0;
        // read up to `self.buffer.len()' compressed bytes into `self.buffer'
        match self.inner.read(self.buffer.as_mut_slice()) {
          // sets self.len to number of bytes actually read `n'
          // should also set self.buf_size?
          Ok(n) => { self.buf_size = n; },
          // got EoF, len == 0, no bytes read?
          Err(ref e) if e.kind == IoErrorKind::EndOfFile => { self.buf_size = 0; },
          // other err, just return it
          Err(e) => { return Err(e); }
        }
        // if we hit EoF, break?
        if self.buf_size <= 0 { break; }
      }

      // while:
      // our position in `buf' is < its length
      // AND
      // our position in `self.buffer' is < its length
      while (dst_offset < buf.len()) && (self.buf_offset < self.buf_size) {

        // we will attempt to decompress remaining amount of bytes in self.buffer
        let mut src_size: size_t = (self.buf_size - self.buf_offset) as size_t;

        let src_buf: &[u8] = self.buffer.slice_from(self.buf_offset);
        let src_buf_ptr: *const c_void = src_buf.as_ptr() as *const c_void;

        let mut dst_size: size_t = (buf.len() - dst_offset) as size_t;
        let dst_buf_ptr: *mut c_void = buf.slice_from_mut(dst_offset).as_mut_ptr() as *mut c_void;

        let maybe_err = unsafe {
          LZ4F_decompress(self.dctx, dst_buf_ptr, &mut dst_size, src_buf_ptr, &mut src_size, ptr::null())
        };
        let len = try!(check_error(maybe_err));
        self.buf_offset += src_size as usize;
        dst_offset += dst_size as usize;

        // no more data expected to decompress
        if len == 0 { self.eof = true; break; }
      }
    }
    Ok(dst_offset)
  }
}
/// decompress the file
/// uses Decompress struct
pub fn decompress2(src: &Path, dst: &Path) -> Result<(), IoError> {

  let src_file = try!(File::open(src));
  let mut dst_file = try!(File::create(dst));

  let mut compressed = Decompressor::new(src_file, None);


  let mut buffer: [u8; 1024] = [0; 1024];
  loop
  {
    let len = try! (compressed.read(&mut buffer));
    if len == 0
    {
      break;
    }
    try!(dst_file.write(&buffer[0..len]));
  }
  Ok(())
}

/// decompress the file
///uses lz4 C functions instead of Decompress struct
pub fn decompress(src: &Path, dst: &Path) -> Result<(), IoError> {

  println!("Decompressing {:?} -> {:?}", src, dst);

  let mut src_file = try!(File::open(src));
  let mut dst_file = try!(File::create(dst));

  let fstat = try!(src_file.stat());
  //println!("got stat");

  let src_buf_size: usize = fstat.size as usize;
  let mut src_buf_size2: size_t = src_buf_size.clone() as size_t;
  //println!("src_buf_size: {:?}", src_buf_size);

  let src_buf: Vec<u8> = try!(src_file.read_to_end());
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
      let out_buf_ptr: *mut c_void = out_buf.as_mut_ptr() as *mut c_void;
      let maybe_ec = LZ4F_decompress(dctx, out_buf_ptr, &mut out_buf_size, src_buf_ptr, &mut src_size_ptr, ptr::null());
      let rv = try!(check_error(maybe_ec));
      out_buf.set_len(out_buf_size as usize);
      dst_file.write(out_buf.as_slice()).unwrap();
      if rv == 0 { break; }
      out_buf.clear();
      src_buf_ptr = src_buf_ptr.offset(src_size_ptr as isize);
    }
    LZ4F_freeDecompressionContext(dctx);
  }


  Ok(())

}
