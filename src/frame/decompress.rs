//! This module contains the `Decompressor` struct which wraps a given `Reader` containing data
//! compressed in the LZ4 Framing format. Whenever `read` is called on a `Decompressor`, the bytes
//! are read out and decompressed before returning the number of bytes read.
//! Additionally, safe wrappers for all of the public decompression functions from `lz4frame.h`
//! are provided.

extern crate libc;
extern crate collections;

use libc::{size_t, c_void};

use std::io::{IoResult, IoError, IoErrorKind};
use std::io::fs::File;

use std::default::Default;

use super::super::liblz4::frame::types::{
    FrameInfo,
    FrameDecompressOptions,
};

use super::super::liblz4::frame::{
    LZ4F_getFrameInfo,
    LZ4F_decompress,
};

use super::structs::{
    Lz4Result,
    FrameContextType,
    Lz4FrameContext,
};

use super::{
    is_error,
    maybe_error,
};

const DEFAULT_BUF_SIZE: usize = 1024;

pub struct Decompressor<R> {
    inner: R,
    dctx: Lz4FrameContext,
    buffer: Vec<u8>,
    buf_size: usize,
    eof: bool,
    buf_offset: usize,
}

/// Decpmpressor struct implementation
impl<R: Reader> Decompressor<R> {
    pub fn new(src: R, buf_size: Option<usize>) -> Lz4Result<Decompressor<R>> {
        let ctx: Lz4FrameContext = try!(create_decompression_context());

        let size: usize = buf_size.unwrap_or(DEFAULT_BUF_SIZE);

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        unsafe { buf.set_len(size); }

        Ok(Decompressor {
            inner: src,
            dctx: ctx,
            buffer: buf,
            buf_size: size,
            eof: false,
            buf_offset: size,
        })
    }
}

impl<R: Reader> Reader for Decompressor<R> {

    /// read the decompressor
    /// if the function reaches EoF, returns an EOF IoError
    /// otherwise, it returns the number of bytes read from the given `buf`.
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
                    // sets self.buf_size to number of bytes actually read `n'
                    Ok(n) => { self.buf_size = n; },
                    // got EoF, len == 0, no bytes read?
                    Err(ref e) if e.kind == IoErrorKind::EndOfFile => { self.buf_size = 0; },
                    // other err, just return it
                    Err(e) => { return Err(e); }
                }
                // if we hit EoF, break
                if self.buf_size <= 0 { break; }
            }

            // while:
            // our position in `buf' is < its length
            // AND
            // our position in `self.buffer' is < its length
            while (dst_offset < buf.len()) && (self.buf_offset < self.buf_size) {

                // we will attempt to decompress remaining amount of bytes in self.buffer
                let mut src_size: usize = self.buf_size - self.buf_offset;

                let src_buf: &[u8] = self.buffer.slice_from(self.buf_offset);

                let mut dst_size: usize = buf.len() - dst_offset;
                let mut dst_buf: &mut [u8] = buf.slice_from_mut(dst_offset);

                match decompress(&mut self.dctx,
                                 dst_buf,
                                 &mut dst_size,
                                 src_buf,
                                 &mut src_size,
                                 None) {
                    Ok(len) => {
                        self.buf_offset += src_size;
                        dst_offset += dst_size;
                        // no more data expected to decompress
                        if len == 0 { self.eof = true; break; }
                    },
                    Err(lz4err) => {
                        return Err(IoError {
                            kind: IoErrorKind::OtherIoError,
                            desc: "lz4 decompress error",
                            detail: Some(lz4err.desc),
                        });
                    }
                }
            }
        }
        Ok(dst_offset)
    }
}

/// Convenient function to decompress a file at the given path `src` to the file at the path `dst`
/// Returns size of decompressed file or an IoError if something failed during decompression.
pub fn decompress_file(src: &Path, dst: &Path, buf_size: Option<usize>) -> IoResult<usize> {

    let src_file = try!(File::open(src));
    let mut dst_file = try!(File::create(dst));

    match Decompressor::new(src_file, None) {
        Ok(ref mut compressed) => {
            let size: usize = buf_size.unwrap_or(DEFAULT_BUF_SIZE);
            let mut buf: Vec<u8> = Vec::with_capacity(size);
            unsafe { buf.set_len(size) };
            loop {
                let bytes_decompressed: usize = match compressed.read(buf.as_mut_slice()) {
                    Ok(n) => n,
                    Err(ref e) if e.kind == IoErrorKind::EndOfFile => { break; },
                    Err(e) => { return Err(e); },
                };
                try!(dst_file.write(buf.slice_to(bytes_decompressed)));
            }
            let fstat = try!(dst_file.stat());
            Ok(fstat.size as usize)
        },
        Err(lz4err) => {
            return Err(IoError {
                kind: IoErrorKind::OtherIoError,
                desc: "lz4 decompress error",
                detail: Some(lz4err.desc),
            });
        },
    }

}


/***********************************
 * Decompression functions
 * *********************************/

/* Resource management */

/// wrapper around `LZ4F_createDecompressionContext()`
pub fn create_decompression_context() -> Lz4Result<Lz4FrameContext> {
    Lz4FrameContext::new(FrameContextType::Decompression)
}

/// This function decodes frame header information, such as `block_size`. It is optional, you could
/// start by calling `decompress()` directly instead. The objective is to extract header information
/// without starting decompression, typically for allocation purposes.
/// The function will work only if `src_buf` points at the beginning of the frame and `src_size` is
/// large enough to decode the whole header (typically, between 7 and 15 bytes).
/// The number of bytes read from `src_buf` will be provided within `src_size` (necessarily <=
/// original value). It is basically the frame header size.
/// You are expected to resume decompression from where it stopped (`src_buff` + `src_size`)
/// 
/// The function result is either a tuple of the decoded `Some(FrameInfo)` and a hint of how many
/// `src_size` bytes `decompress()` expects for the next call, or `None` and an `Lz4Error`.
pub fn get_frame_info(dctx: &mut Lz4FrameContext,
                      src_buf: &[u8],
                      src_size: &mut usize) -> (Option<FrameInfo>, Lz4Result<usize>) {
    let mut finfo: FrameInfo = Default::default();
    let src_ptr: *const c_void = src_buf.as_ptr() as *const c_void;
    let mut src_size_t: size_t = src_size.clone() as size_t;
    let err = unsafe {
        LZ4F_getFrameInfo(dctx.ctx, &mut finfo, src_ptr, &mut src_size_t)
    };
    *src_size = src_size_t as usize;
    if is_error(err) {
        (None, maybe_error(err))
    } else {
        (Some(finfo), maybe_error(err))
    }
}

/// Call this function repetitively to regenerate data compressed within `src_buf`. The function
/// will attempt to decode `src_size` bytes from `src_buf` into `dst_buf` of maximum size `dst_size`
///
/// The number of bytes regenerated into `dst_buf` will be provided within `dst_size` (necessarily
/// <= original value).
/// The number of bytes read from `src_buf` will be provided within `src_size` (necessarily, <=
/// original value).
/// If number of bytes read is < number of bytes provided, then decomrpession operation is not
/// completed. It typically happens when `dst_buf` is not large enough to contain all decoded data.
/// `decompress()` must be called again, starting from where it stopped (`src_buf` + `src_size`).
/// The underlying `LZ4F_decompress()` function will check this condition and refuse to continue
/// if it is not respected.
///
/// `dst_buf` is supposed to be flushed between each call to the function, since its content will be
/// overweitten. `dst*` options can be changed at will with each consecutive call to the function.
///
/// The function result is a hint of how many `src_size` bytes `decompress()` expects for the next
/// call. Schematically, it's the size of the current (or remaining) compressed block + header of 
/// the next block. Respecting the hing provides some boost to performance, since it does skip
/// intermediate buffers. This is just a hint though, you can always provide any `src_size` you
/// want.
/// When a frame is fully decoded, the function result will be 0 (no more data expected).
/// If decompression failed, the result will be an `Lz4Error`.
///
/// After a frame is fully decoded, `dctx` can be used again to decompress another frame.
pub fn decompress(dctx: &mut Lz4FrameContext,
                  dst_buf: &mut [u8],
                  dst_size: &mut usize,
                  src_buf: &[u8],
                  src_size: &mut usize,
                  decompress_opts: Option<&FrameDecompressOptions>) -> Lz4Result<usize> {

    let def_opts: FrameDecompressOptions = Default::default();
    let opts: &FrameDecompressOptions = match decompress_opts {
        Some(o) => o,
        None => &def_opts,
    };

    let mut dst_size_t: size_t = dst_size.clone() as size_t;
    let mut src_size_t: size_t = src_size.clone() as size_t;

    let dst_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let src_ptr: *const c_void = src_buf.as_ptr() as *const c_void;

    let err = unsafe {
        LZ4F_decompress(dctx.ctx, dst_ptr, &mut dst_size_t, src_ptr, &mut src_size_t, opts)
    };

    *dst_size = dst_size_t as usize;
    *src_size = src_size_t as usize;

    maybe_error(err)
}
