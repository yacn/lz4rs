#![allow(unstable)]
extern crate lz4rs;

use std::os;

fn main() {
    let v = lz4rs::version();

    println!("version: {}", v);

    let suffix = ".lz4";

    for arg in os::args().tail().iter() {
        //lz4rs::compress(&Path::new(arg), &Path::new(&(arg.to_string() + suffix))).unwrap();
        lz4rs::frame::decompress2(&Path::new(arg), &Path::new(&arg[0..arg.len()-suffix.len()])).unwrap();
    }
}