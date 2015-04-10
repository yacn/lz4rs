//! This module contains all the associated types, structs, methods, and functions for use with the
//! LZ4 Framing Format.

extern crate libc;
extern crate collections;

use super::liblz4::frame::LZ4F_isError;
use super::liblz4::frame::types::FrameErrorCode;


pub use self::structs::{
    Lz4Error,
    Lz4Result,
    FrameContextType,
    Lz4FrameContext,
};

pub mod structs;
pub mod compress;
pub mod decompress;


/**************************************
 * Error management
 * ************************************/

/// checks if the input is an LZ4 error code
pub fn is_error(code: FrameErrorCode) -> bool {
    let result: usize = unsafe { LZ4F_isError(code) as usize };
    result != 0
}

/// Checks whether the given code is an error code. If not (i.e. when it's some number of bytes
/// compressed/decompressed), returns the code as a usize. Else, returns an Lz4Error with the
/// message associated with the code.
pub fn maybe_error(code: FrameErrorCode) -> Lz4Result<usize> {
    if is_error(code) {
        Err(Lz4Error::new(code))
    } else {
        Ok(code as usize)
    }
}


/// Simple tests that the Compressor/Decompressor work as expected.
mod basic_functionality_tests {
    use std::io::MemReader;
    use super::compress::Compressor;
    use super::decompress::Decompressor;

    /// Tests to ensure that we can compress some data and then receive the same data back when
    /// decompressing
    #[test]
    fn it_works() {
        let mut compressor: Compressor<Vec<u8>> = Compressor::default(Vec::new()).ok().unwrap();
        let data: &[u8] = b"This is a test\nA what?\nA test\nA what?\nA test\nOh a test\n";
        compressor.write(data).unwrap();
        let (v, result): (Vec<u8>, Lz4Result<usize>) = compressor.done();
        result.ok().unwrap();

        let readr: MemReader = MemReader::new(v);
        let mut decompressor: Decompressor<MemReader> = Decompressor::new(readr, None).ok()
                                                                                      .unwrap();
        let mut buf: [u8; 1024] = [0; 1024];
        let bytes_decompressed: usize = decompressor.read(&mut buf).unwrap();
        assert_eq!(data, buf.slice_to(bytes_decompressed));
    }
}
