use log::{Metadata, Record};
use std::fmt::Write;

extern "C" {
	fn px4_log_modulename(level: i32, module: *const u8, fmt: *const u8, ...);
	fn px4_log_raw(level: i32, fmt: *const u8, ...);
}

#[doc(hidden)]
pub enum LogLevel {
	Debug = 0,
	Info = 1,
	Warn = 2,
	Error = 3,
	Panic = 4,
}

#[doc(hidden)]
pub fn log_raw(level: LogLevel, message: &str) {
	unsafe {
		px4_log_raw(
			level as i32,
			"%.*s\0".as_ptr(),
			message.len() as i32,
			message.as_ptr(),
		);
	}
}

/// Print output without any decoration.
///
/// The equivalent of `PX4_INFO_RAW` in C and C++.
///
/// ## Example
///
/// ```
/// # #[macro_use] extern crate px4;
/// # #[no_mangle] fn px4_log_raw() {}
/// # fn main() {
/// info_raw!("Hello World!\n");
/// info_raw!("Hello {}!\n", "World");
/// # }
/// ```
#[macro_export]
macro_rules! info_raw {
	($arg:expr) => (
		$crate::log_raw($crate::LogLevel::Info, $arg)
	);
	($($arg:tt)+) => (
		$crate::log_raw($crate::LogLevel::Info, &format!($($arg)+))
	);
}

struct Px4Logger;

impl log::Log for Px4Logger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= log::Level::Info
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}

		let level = match record.level() {
			log::Level::Error => LogLevel::Error,
			log::Level::Warn => LogLevel::Warn,
			log::Level::Info => LogLevel::Info,
			log::Level::Debug => LogLevel::Debug,
			log::Level::Trace => LogLevel::Debug,
		};

		// We allocate both nul-terminated strings as one single String, to
		// save on heap allocations.
		let target = record.target();
		let s = format!("{}\0{}\0", target, record.args());
		let (module, message) = s.as_bytes().split_at(target.len() + 1);

		unsafe {
			px4_log_modulename(
				level as i32,
				module.as_ptr(),
				"%s\0".as_ptr(),
				message.as_ptr(),
			);
		}
	}

	fn flush(&self) {}
}

static LOGGER: Px4Logger = Px4Logger;

pub unsafe fn init(modulename: &'static [u8]) {
	if log::set_logger(&LOGGER).is_ok() {
		log::set_max_level(log::LevelFilter::Info);
		std::panic::set_hook(Box::new(move |info: &std::panic::PanicInfo| {
			let payload: &str = if let Some(s) = info.payload().downcast_ref::<&'static str>() {
				s
			} else if let Some(s) = info.payload().downcast_ref::<String>() {
				&s
			} else {
				"[unknown]"
			};
			let mut message = String::new();
			let thread = std::thread::current();
			if let Some(name) = thread.name() {
				write!(message, "thread '{}' ", name).unwrap();
			}
			write!(message, "panicked at '{}'", payload).unwrap();
			if let Some(loc) = info.location() {
				write!(message, ", {}", loc).unwrap();
			}
			message.push('\0');
			px4_log_modulename(
				LogLevel::Panic as i32,
				modulename.as_ptr(),
				"%s\0".as_ptr(),
				message.as_ptr(),
			);
		}));
	}
}
