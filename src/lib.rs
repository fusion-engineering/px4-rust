extern crate log;
use std::ffi::CStr;
use std::os::raw::c_char;

mod logging;

pub use crate::logging::{log_raw, LogLevel};

#[macro_export]
macro_rules! info_raw {
	($($arg:tt)+) => (
		$crate::log_raw($crate::LogLevel::Info, &format!($($arg)+))
	)
}

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

#[macro_export]
macro_rules! px4_module {
	($main:expr) => {
		#[no_mangle]
		pub extern "C" fn px4_module_main(argc: u32, argv: *mut *mut u8) -> i32 {
			$crate::_run(concat!(module_path!(), "\0").as_bytes(), argc, argv, $main)
		}
	};
}
