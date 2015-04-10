//! This module contains structs and trait implementations needed for the core module.

use std::fmt;

const MAJOR: usize = 100*100;
const MINOR: usize = 100;

#[derive(Show)]
pub struct Lz4Version {
    pub major: usize,
    pub minor: usize,
    pub release: usize,
    raw: usize,
}

impl Lz4Version {
    pub fn new(version: usize) -> Lz4Version{
        let mut v: usize = version;
        let major: usize = v / MAJOR;
        v -= major*MAJOR;
        let minor: usize = v / MINOR;
        v -= minor*MINOR;
        Lz4Version {
            major: major,
            minor: minor,
            release: v,
            raw: version,
        }
    }
}

impl fmt::String for Lz4Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}.{}.{}", self.major, self.minor, self.release)
    }
}