extern crate gcc;

use std::default::Default;

fn main() {

	gcc::compile_library("liblz4.a", &Default::default(), &[
		"ext/lz4/lib/lz4.c",
		"ext/lz4/lib/lz4frame.c",
		"ext/lz4/lib/lz4hc.c",
		"ext/lz4/lib/xxhash.c",]);
}