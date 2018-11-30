extern crate log;
extern crate px4_macros;

use std::ffi::CStr;
use std::os::raw::c_char;

pub mod uorb;
mod logging;

pub use crate::logging::{log_raw, LogLevel};
pub use px4_macros::px4_module_main;

#[doc(hidden)]
pub fn _run<F>(modulename: &'static [u8], argc: u32, argv: *mut *mut u8, f: F) -> i32
where
	F: Fn(&[&str]) -> i32 + std::panic::UnwindSafe,
{
	unsafe { logging::init(modulename) };
	std::panic::catch_unwind(move || {
		let mut args = Vec::with_capacity(argc as usize);
		for i in 0..argc {
			args.push(
				unsafe { CStr::from_ptr(*argv.offset(i as isize) as *const c_char) }
					.to_str()
					.unwrap_or_else(|_| panic!("Invalid UTF-8 in arguments.")),
			);
		}
		f(&args)
	}).unwrap_or(-1)
}
