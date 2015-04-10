//! This is a basic example showing how the `lz4rs` module can be used.

#![allow(unstable)]
extern crate lz4rs;

use std::os;
use std::io::IoErrorKind::EndOfFile;

use lz4rs::frame::decompress::decompress_file;
use lz4rs::frame::compress::compress_file;

const EXTENSION: &'static str = ".lz4";

fn main() {
    println!("LZ4 version {}", lz4rs::version());

    for arg in os::args().tail().iter() {
        if arg.ends_with(EXTENSION) {
            let src: Path = Path::new(arg);
            let dst: Path = Path::new(arg.replace(EXTENSION, ""));
            match decompress_file(&src, &dst, None) {
                Ok(bytes) => {
                    println!("Decompressed {:?} into {} bytes", arg, bytes);
                },
                Err(ref e) if e.kind == EndOfFile => {
                    println!("finished decompressing");
                },
                Err(e) => { println!("{:?}", e); },
            }
        } else {
            let src: Path = Path::new(arg);
            let mut dst_name: String = arg.to_string();
            dst_name.push_str(EXTENSION);
            let dst: Path = Path::new(dst_name);
            match compress_file(&src, &dst, None) {
                Ok(bytes) => {
                    println!("compressed {:?} into {} bytes", arg, bytes);
                },
                Err(ref e) if e.kind == EndOfFile => {
                    println!("finished compressing");
                },
                Err(e) => { println!("{:?}", e); },
            }
        }
    }
}
