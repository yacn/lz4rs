extern crate lz4rs;

fn main() {
	let v = lz4rs::version();

	println!("version: {}", v);
}