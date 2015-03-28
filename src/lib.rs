extern crate libc;

pub fn version() -> usize {
	let result = unsafe { LZ4_versionNumber() };
	return result as usize;
}

extern {

	fn LZ4_versionNumber() -> libc::c_int;

}