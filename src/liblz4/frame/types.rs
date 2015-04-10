//! Rust wrappers around the types, enums, and structs defined in `lz4frame.h`

#![allow(unstable)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

extern crate libc;

use libc::{c_uint, c_int, size_t, c_char, c_void};

use std::ptr;
use std::default::Default;
use std::ffi::CString;

pub type FrameErrorCode = size_t;

// TODO: Update Lz4Errors to be more like IoErrors
pub enum FrameErrorKind {
    OkNoError = 0,
    Generic = 1,
    InvalidMaxBlockSize = 2,
    InvalidBlockMode = 3,
    InvalidContentChecksumFlag = 4,
    InvalidCompressionLevel = 5,
    FailedAllocation = 6,
    SrcSizeTooLarge = 7,
    DstMaxSizeTooSmall = 8,
    WrongFrameSize = 9,
    UnknownFrameType = 10,
    FailedDecompression = 11,
    InvalidChecksum = 12,
}

pub type Context = *mut c_void;

#[derive(Show)]
#[repr(C)]
pub enum BlockSize {
    Default = 0,
    Max64KB = 4,
    Max256KB = 5,
    Max1MB = 6,
    Max4MB = 7,
}

#[derive(Show)]
#[repr(C)]
pub enum BlockMode {
    Linked = 0,
    Independent,
}

#[derive(Show)]
#[repr(C)]
pub enum ContentChecksum {
    Disabled = 0,
    Enabled,
}

#[derive(Show)]
#[repr(C)]
pub enum FrameType {
    Default = 0,
    SkippableFrame,
}

#[derive(Show)]
#[repr(C)]
pub struct FrameInfo {
    pub block_size_id: BlockSize,
    pub block_mode: BlockMode,
    pub content_checksum_flag: ContentChecksum,
    pub frame_type: FrameType,
    pub content_size: u64,
    pub reserved: [c_uint; 2],
}

impl FrameInfo {
    fn new(bsize: BlockSize,
           bmode: BlockMode,
           checksum_flag: bool,
           ftype: FrameType,
           content_size: usize) -> FrameInfo {
        FrameInfo {
            block_size_id: bsize,
            block_mode: bmode,
            content_checksum_flag: if checksum_flag { ContentChecksum::Enabled }
                                   else { ContentChecksum::Disabled },
            frame_type: ftype,
            content_size: content_size as u64,
            reserved: [0; 2],
        }
    }
}

impl Default for FrameInfo {
    fn default() -> FrameInfo {
        FrameInfo {
            block_size_id: BlockSize::Default,
            block_mode: BlockMode::Linked,
            content_checksum_flag: ContentChecksum::Disabled,
            frame_type: FrameType::Default,
            content_size: 0,
            reserved: [0; 2],
        }
    }
}


#[repr(C)]
pub struct FramePreferences {
    pub frame_info: FrameInfo,
    pub compression_level: c_uint,
    pub auto_flush: c_uint,
    pub reserved: [c_uint; 4],
}

impl FramePreferences {
    fn new(finfo: FrameInfo, compress_lvl: usize, auto_flush: bool) -> FramePreferences {
        FramePreferences {
            frame_info: finfo,
            compression_level: if compress_lvl > 16 { 16 } else { compress_lvl as c_uint },
            auto_flush: if auto_flush { 1 } else { 0 },
            reserved: [0; 4],
        }
    }
}

impl Default for FramePreferences {
    fn default() -> FramePreferences {
        FramePreferences {
            frame_info: Default::default(),
            compression_level: 0,
            auto_flush: 0,
            reserved: [0; 4],
        }
    }
}

#[repr(C)]
pub struct FrameCompressOptions {
    /// 1 == src content will remain available on future calls to LZ4F_compress(); avoid saving
    /// src content within tmp buffer as future dictionary
    pub stable_src: c_uint,
    pub reserved: [c_uint; 3],
}

impl Default for FrameCompressOptions {
    fn default() -> FrameCompressOptions {
        FrameCompressOptions {
            stable_src: 0,
            reserved: [0; 3],
        }
    }
}

impl FrameCompressOptions {
    fn new(src_stable: bool) -> FrameCompressOptions {
        FrameCompressOptions {
            stable_src: if src_stable { 1 } else { 0 },
            reserved: [0; 3],
        }
    }
}

#[repr(C)]
pub struct FrameDecompressOptions {
    /// guarantee that decompresed data will still be there on next function calls (avoid storage
    /// into tmp buffers)
    stable_dst: c_uint,
    reserved: [c_uint; 3],
}

impl Default for FrameDecompressOptions {
    fn default() -> FrameDecompressOptions {
        FrameDecompressOptions {
            stable_dst: 0,
            reserved: [0; 3],
        }
    }
}

impl FrameDecompressOptions {
    fn new(dst_stable: bool) -> FrameDecompressOptions {
        FrameDecompressOptions {
            stable_dst: if dst_stable { 1 } else { 0 },
            reserved: [0; 3],
        }
    }
}
