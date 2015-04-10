//! This module contains the `Compressor` struct which wraps a given `Writer`, compressing any
//! data written to it into an LZ4 Frame. 
//! Additionally, safe wrappers for all of the public compression functions from `lz4frame.h`
//! are provided

extern crate libc;
extern crate collections;

use libc::{size_t, c_void};

use std::io::fs::File;
use std::io::{IoResult, IoError, IoErrorKind};
use std::default::Default;
use std::cmp::min;

use super::super::liblz4::frame::types::{
    FramePreferences,
    FrameCompressOptions,
};

use super::super::liblz4::frame::{
    LZ4F_compressFrameBound,
    LZ4F_compressFrame,
    LZ4F_compressBegin,
    LZ4F_compressBound,
    LZ4F_compressUpdate,
    LZ4F_flush,
    LZ4F_compressEnd,
};

use super::structs::{
    Lz4Result,
    FrameContextType,
    Lz4FrameContext,
};

use super::{
    maybe_error,
};

/// How big the internal buffer in `Compressor` should be by default.
const DEFAULT_BUF_SIZE: usize = 1024;

pub struct Compressor<W> {
    inner: W,
    cctx: Lz4FrameContext,
    buffer: Vec<u8>,
    buf_size: usize,
    opts: FrameCompressOptions,
    prefs: FramePreferences,
}

impl<W: Writer> Compressor<W> {
    /// Creates a new `Compressor` wrapping the given `Writer` `dst`. If any of `prefs`, `buf_size`,
    /// or `opts` is provided, the `Compressor` is created with those options configured. Otherwise,
    /// their defaults are used.
    pub fn new(mut dst: W,
               prefs: Option<FramePreferences>,
               buf_size: Option<usize>,
               opts: Option<FrameCompressOptions>) -> Lz4Result<Compressor<W>> {
        let mut ctx: Lz4FrameContext = try!(create_compression_context());

        let prefs: FramePreferences = prefs.unwrap_or_else(Default::default);
        let opts: FrameCompressOptions = opts.unwrap_or_else(Default::default);

        let s: usize = buf_size.unwrap_or(DEFAULT_BUF_SIZE);
        let size: usize = compress_bound(s, Some(&prefs));
        let mut buf: Vec<u8> = Vec::with_capacity(size);

        // add frame header
        let len: usize = try!(compress_begin(&mut ctx, buf.as_mut_slice(), size, Some(&prefs)));
        unsafe { buf.set_len(len); }
        dst.write(buf.as_slice());
        Ok(Compressor {
            inner: dst,
            cctx: ctx,
            buffer: buf,
            buf_size: size,
            opts: opts,
            prefs: prefs,
        })
    }

    /// Creates a `Compressor` with all default options and preferences set.
    pub fn default(mut dst: W) -> Lz4Result<Compressor<W>> {
        let mut ctx: Lz4FrameContext = try!(create_compression_context());

        let prefs: FramePreferences = Default::default();
        let opts: FrameCompressOptions = Default::default();
        let size: usize = compress_bound(DEFAULT_BUF_SIZE, Some(&prefs));
        let mut buf: Vec<u8> = Vec::with_capacity(size);

        let len: usize = try!(compress_begin(&mut ctx, buf.as_mut_slice(), size, Some(&prefs)));
        unsafe {buf.set_len(len); }
        dst.write(buf.as_slice());
        Ok(Compressor {
            inner: dst,
            cctx: ctx,
            buffer: buf,
            buf_size: size,
            opts: opts,
            prefs: prefs,
        })
    }

    /// Unwraps this `Compressor`, returning underlying Writer
    fn into_inner(self) -> W { self.inner }

    /// Properly finishes the frame being compressed by calling `compress_end` and writing the
    /// result to the inner `Writer`
    pub fn end(&mut self) -> Lz4Result<usize> {
        let len: usize = try!(compress_end(&mut self.cctx,
                                           self.buffer.as_mut_slice(),
                                           self.buf_size,
                                           Some(&self.opts)));
        unsafe { self.buffer.set_len(len) }
        self.inner.write(self.buffer.as_slice());
        Ok(len)
    }

    /// finishes the frame being compressed and returns the inner `Writer` for further use.
    pub fn done(mut self) -> (W, Lz4Result<usize>) {
        let end_res: Lz4Result<usize> = self.end();
        (self.into_inner(), end_res)
    }
}

impl<W: Writer> Writer for Compressor<W> {
    /// Implementation of `write` for `Compressor`. Whenever `write` is called, the given byte buf
    /// is compressed and written to the inner `Writer` inside `Compressor`. It uses
    /// `compress_update` to compress the bytes before writing.
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {

        let mut buf_offset: usize = 0;

        while buf_offset < buf.len() {
            let rem_space: usize = buf.len() - buf_offset;
            let size: usize = min(self.buf_size, rem_space);
            match compress_update(&mut self.cctx,
                                  self.buffer.as_mut_slice(),
                                  self.buf_size,
                                  buf.slice_from(buf_offset),
                                  size,
                                  Some(&self.opts)) {
                Ok(len) => {
                    unsafe { self.buffer.set_len(len); }
                    try!(self.inner.write(self.buffer.as_slice()));
                    buf_offset += size;
                },
                Err(lz4err) => {
                    return Err(IoError {
                        kind: IoErrorKind::OtherIoError,
                        desc: "lz4 compress error",
                        detail: Some(lz4err.desc),
                    });
                },
            }
        }
        Ok(())
    }

    /// Implementation of `flush`, using the wrapped `flush` function. Flushes any data from the
    /// `Compressor`'s compression context, writing it to the inner `Writer`.
    fn flush(&mut self) -> IoResult<()> {
        loop {
            match flush(&mut self.cctx,
                        self.buffer.as_mut_slice(),
                        self.buf_size,
                        Some(&self.opts)) {
                Ok(len) => {
                    if len == 0 { break; }
                    unsafe { self.buffer.set_len(len); }
                    self.inner.write(self.buffer.as_slice());
                },
                Err(lz4err) => {
                    return Err(IoError { 
                        kind: IoErrorKind::OtherIoError,
                        desc: "lz4 flush error",
                        detail: Some(lz4err.desc),
                    });
                },
            }
        }
        self.inner.flush()
    }
}

pub fn compress_file(src: &Path, dst: &Path, buf_size: Option<usize>) -> IoResult<usize> {
    let mut src_file = try!(File::open(src));
    let mut dst_file = try!(File::create(dst));

    let mut compressor = match Compressor::new(dst_file, None, None, None) {
        Ok(c) => c,
        Err(lz4err) => {
            return Err(IoError {
                kind: IoErrorKind::OtherIoError,
                desc: "lz4 compress error",
                detail: Some(lz4err.desc)
            });
        }
    };
    let size: usize = buf_size.unwrap_or(DEFAULT_BUF_SIZE);
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    unsafe { buf.set_len(size) };
    loop {
        let bytes_read = match src_file.read(buf.as_mut_slice()) {
            Ok(n) => n,
            Err(ref e) if e.kind == IoErrorKind::EndOfFile => { break; },
            Err(e) => { return Err(e); },
        };
        try!(compressor.write(buf.slice_to(bytes_read)));
    }
    let (compressed, _) = compressor.done();
    let fstat = try!(compressed.stat());
    Ok(fstat.size as usize)
}

/***********************************
 * Simple compression function
 * *********************************/


/// determine the minimum value necessary for dstMaxSize
pub fn compress_frame_bound(src_size: usize,
                            prefs: Option<&FramePreferences>) -> usize {
    let def_prefs: FramePreferences = Default::default();
    let prefs: &FramePreferences = match prefs {
        Some(p) => p,
        None => &def_prefs,
    };
    let bound: size_t = unsafe { LZ4F_compressFrameBound(src_size as size_t, prefs) };
    bound as usize
}

/// Compress an entire `src_buf` into a valid LZ4 frame, as defined by specification v1.5
/// The most important rule is that `dst_buf` MUST be large enough (`dst_max_size`) to ensure
/// compression completion even in worst case. You can get the minimum value of `dst_max_size`
/// by using `compress_frame_bound()`.
/// If this condition is not respected, `compress_frame()` will fail with an `Lz4Error`.
/// Providing `None` for `prefs` will result in default preferences being used.
/// The result of the function is the number of bytes written into `dst_buf`.
pub fn compress_frame(dst_buf: &mut [u8],
                      dst_max_size: usize,
                      src_buf: &[u8],
                      prefs: Option<&FramePreferences>) -> Lz4Result<usize> {
    let src_size: size_t = src_buf.len() as size_t;
    let dst_max_size: size_t = dst_max_size as size_t;
    let dst_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let src_ptr: *const c_void = src_buf.as_ptr() as *const c_void;
    let def_prefs: FramePreferences = Default::default();
    let prefs: &FramePreferences = match prefs {
        Some(p) => p,
        None    => &def_prefs,
    };
    let err = unsafe {
        LZ4F_compressFrame(dst_ptr, dst_max_size, src_ptr, src_size, prefs)
    };
    maybe_error(err)
}


/**********************************
 * Advanced compression functions
 * ********************************/

/* Resource Management */

/// wrapper around `LZ4F_createCompressionContext()`
pub fn create_compression_context() -> Lz4Result<Lz4FrameContext> {
    Lz4FrameContext::new(FrameContextType::Compression)
}

/* Compression */

/// Writes the frame header into `dst_buf`. `dst_buf` must be large enough to accomadate a header
/// (`dst_max_size`). Maximum header size is 15 bytes. Providing `None` for `prefs` results in the
/// default preferences being used. The result is either the number of bytes written into `dst_buf`
/// for the header or an `Lz4Error`.
pub fn compress_begin(cctx: &mut Lz4FrameContext,
                      dst_buf: &mut [u8],
                      dst_max_size: usize,
                      prefs: Option<&FramePreferences>) -> Lz4Result<usize> {
    let def_prefs: FramePreferences = Default::default();
    let prefs: &FramePreferences = match prefs {
        Some(p) => p,
        None => &def_prefs,
    };
    let dst_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let dst_max_size: size_t = dst_max_size as size_t;
    let err = unsafe {
        LZ4F_compressBegin(cctx.ctx, dst_ptr, dst_max_size, prefs)
    };
    maybe_error(err)
}

/// Provides the minimum size of the destination buffer given `src_size` to handle worst case
/// situations. Providing `None` for `prefs` results in the default preferences being used.
/// Note that different preferences wil produce different results.
pub fn compress_bound(src_size: usize, prefs: Option<&FramePreferences>) -> usize {
    let def_prefs: FramePreferences = Default::default();
    let prefs: &FramePreferences = match prefs {
        Some(p) => p,
        None => &def_prefs,
    };
    let bound = unsafe { LZ4F_compressBound(src_size as size_t, prefs) };
    bound as usize
}

/// `compress_update()` can be called repetitively to compress as much data as necessary. The most
/// important rule is that `dst_buf` MUST be large enough (`dst_max_size`) to ensure compression
/// completion even in worst case.
/// If this condition is not respected, `compress_update()` will fail with an `Lz4Error`. You can
/// get the minimum value of `dst_max_size` by using `compress_bound()`. If `None` is provided
/// for `compress_opts`, the default compression options are used. 
/// The result of the function is the number of bytes written into `dst_buf`. It can be zero,
/// meaning input data was just buffered.
pub fn compress_update(cctx: &mut Lz4FrameContext,
                       dst_buf: &mut [u8],
                       dst_max_size: usize,
                       src_buf: &[u8],
                       src_size: usize,
                       compress_opts: Option<&FrameCompressOptions>) -> Lz4Result<usize> {
    let def_opts = Default::default();
    let compress_opts: &FrameCompressOptions = match compress_opts {
        Some(c) => c,
        None => &def_opts,
    };
    let dst_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let dst_max_size: size_t = dst_max_size as size_t;
    let src_ptr: *const c_void = src_buf.as_ptr() as *const c_void;
    let src_size: size_t = src_size as size_t;
    let err = unsafe {
        LZ4F_compressUpdate(cctx.ctx, dst_ptr, dst_max_size, src_ptr, src_size, compress_opts)
    };
    maybe_error(err)
}

/// Should you need to generate compressed data immediately, without waiting for the current block
/// to be filled, you can call `flush()`, which will immediately compress any remaining data
/// buffered within `cctx`.
/// Note that `dst_max_size` must be large enough to ensure the operation will be successful.
/// If `None` is provided for `compress_opts`, the default compression options will be used.
/// The result of the function is either the number of bytes written into `dst_buffer` 
/// (which can be zero, meaning there was no data left within `cctx`) or an `Lz4Error`
pub fn flush(cctx: &mut Lz4FrameContext,
             dst_buf: &mut [u8],
             dst_max_size: usize,
             compress_opts: Option<&FrameCompressOptions>) -> Lz4Result<usize> {
    let def_opts: FrameCompressOptions = Default::default();
    let compress_opts: &FrameCompressOptions = match compress_opts {
        Some(c) => c,
        None => &def_opts,
    };
    let dst_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let dst_max_size: size_t = dst_max_size as size_t;
    let err = unsafe {
        LZ4F_flush(cctx.ctx, dst_ptr, dst_max_size, compress_opts)
    };
    maybe_error(err)
}

/// When you want to properly finish the compressed frame, just call `compress_end()`. It will
/// flush whatever data remained within `cctx` (like `flush()`) but will also properly finalize the
/// frame with an `endMark` and a checksum. If `None` is provided for `compress_opts`, the default
/// compression options will be used.
/// The result of the function is either the number of bytes written into `dst_buf`
/// (necessarily >= 4 (`endMark` size)) or an `Lz4Error`.
pub fn compress_end(cctx: &mut Lz4FrameContext,
                    dst_buf: &mut [u8],
                    dst_max_size: usize,
                    compress_opts: Option<&FrameCompressOptions>) -> Lz4Result<usize> {
    let def_opts: FrameCompressOptions = Default::default();
    let compress_opts: &FrameCompressOptions = match compress_opts {
        Some(c) => c,
        None => &def_opts,
    };
    let dst_ptr: *mut c_void = dst_buf.as_mut_ptr() as *mut c_void;
    let dst_max_size: size_t = dst_max_size as size_t;
    let err = unsafe {
        LZ4F_compressEnd(cctx.ctx, dst_ptr, dst_max_size, compress_opts)
    };
    maybe_error(err)
}
