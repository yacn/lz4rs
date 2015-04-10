//! This module contains wrappers around the functions inside `lz4.h`

pub use self::structs::Lz4Version;

use super::liblz4::core::LZ4_versionNumber;

pub mod structs;

pub fn version() -> Lz4Version {
    let result = unsafe { LZ4_versionNumber() };
    Lz4Version::new(result as usize)
}
