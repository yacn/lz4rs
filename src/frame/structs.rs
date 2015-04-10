//! This module contains various structs needed by both the compressor and decompressor modules
//! as well as implementations of various necessary traits.

use libc;
use libc::c_char;

use collections::slice;

use std::mem;
use std::str;
use std::ptr;
use std::ops::{Deref, DerefMut};

use super::super::liblz4::frame::types::{
    Context,
    FrameErrorCode,
};

use super::super::liblz4::frame::{
    LZ4F_VERSION,
    LZ4F_getErrorName,
    LZ4F_createCompressionContext,
    LZ4F_freeCompressionContext,
    LZ4F_createDecompressionContext,
    LZ4F_freeDecompressionContext,
};

use super::is_error;

/// Convenient wrapper for returning Results
pub type Lz4Result<T> = Result<T, Lz4Error>;

/// Contains a description message of what the error is.
pub struct Lz4Error {
    pub desc: String, 
}

impl Lz4Error {
    /// Given an error code, retrieves the associated error string and wraps it in an `Lz4Error`
    /// struct.
    pub fn new(code: FrameErrorCode) -> Lz4Error {
        let emsg = unsafe { get_error_string(code) };
        Lz4Error { desc: emsg }
    }
}

/// Retrieves string representation of given error code. 
unsafe fn get_error_string(code: FrameErrorCode) -> String {
    let emsg_ptr: *const c_char = LZ4F_getErrorName(code);
    str_from_ptr(emsg_ptr)
}

/// Converts a pointer to a C string to a String buffer
unsafe fn str_from_ptr(ptr: *const c_char) -> String {
    let len: usize = (libc::strlen(ptr) as usize) + 1;
    let char_slice: &[c_char] = slice::from_raw_buf(&ptr, len);
    let byte_slice: &[u8] = mem::transmute(char_slice);
    str::from_utf8(byte_slice).unwrap().to_string()
}

/// Enum to indicate which type of compression context is being used inside an `Lz4FrameContxt`
#[derive(Copy)]
pub enum FrameContextType {
    Compression,
    Decompression,
}

/// Wrapper around LZ4 contexts (compression and decompression)
pub struct Lz4FrameContext {
    pub ctx: Context,
    ty: FrameContextType,
}

impl Lz4FrameContext {
    /// Creates a new `Lz4FrameContext` based on which `FrameContextType` is given.
    pub fn new(t: FrameContextType) -> Lz4Result<Lz4FrameContext> {
        let mut ctx: Context = ptr::null_mut();
        let err = match t {
            FrameContextType::Compression => {
                unsafe { LZ4F_createCompressionContext(&mut ctx, LZ4F_VERSION) }
            },
            FrameContextType::Decompression => {
                unsafe { LZ4F_createDecompressionContext(&mut ctx, LZ4F_VERSION) }
            },
        };

        if is_error(err) {
            Err(Lz4Error::new(err))
        } else {
            Ok(Lz4FrameContext { ctx: ctx, ty: t })
        }
    }
}

/// Implementation of Deref for an Lz4FrameContext for a convienent way to
/// access the underlying context.
impl Deref for Lz4FrameContext {
    type Target = Context;

    fn deref<'a>(&'a self) -> &'a Context {
        &self.ctx
    }
}

/// Mutable implementation of Deref for an Lz4FrameContext for a convienent way to
/// change the underlying context if needed.
impl DerefMut for Lz4FrameContext {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Context {
        &mut self.ctx
    }
}

/// Implements drop to ensure the underlying context is free'd properly.
impl Drop for Lz4FrameContext {
    fn drop(&mut self) {
        match self.ty {
            FrameContextType::Compression => {
                unsafe { LZ4F_freeCompressionContext(self.ctx); }
            },
            FrameContextType::Decompression => {
                unsafe { LZ4F_freeDecompressionContext(self.ctx); }
            },
        }
    }
}
