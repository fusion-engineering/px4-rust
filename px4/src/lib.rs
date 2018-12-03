//! # PX4 bindings for Rust
//!
//! This crate provides the framework to make dynamically loadable PX4 modules
//! in Rust. Right now, it provides bindings for the two most important APIs:
//! Logging and uORB. It also provides the entry point for your module, and
//! handles panics on the main thread of the module.
//!
//! See the
//! [`example` directory](https://github.com/dronesforwork/px4-rust/tree/master/example)
//! for an example module.
//!
//! ## Compiling and running
//!
//! To build a PX4 module in Rust, create a crate as you would for any other
//! application binary, and then add the following to your Cargo.toml:
//!
//! ```text
//! [lib]
//! crate-type = ["cdylib"]
//! path = "src/module.rs"
//! ```
//!
//! This will turn your program into a loadable module instead of a standalone
//! application. The resulting file will be called `lib<name>.so`, which you
//! can manually rename to `<name>.px4mod` if you want.
//!
//! To run your module, use the
//! [`dyn`](https://dev.px4.io/en/middleware/modules_command.html#dyn)
//! PX4 command. Give it the full path name, followed by any arguments to your
//! module. Note that `dyn` will *not* reload your file if you run it again.
//! If you want to run a changed version of your module, you'll either need to
//! restart PX4, or move/rename the file.
//!
//! ## Entry point
//!
//! Mark your entry function with `#[px4_module_main]`. The boilerplate code
//! needed to set up the environment and export the function under the right
//! name is then inserted automatically.
//!
//! Your main function should take a `&[&str]` as argument, and *may* return a
//! `i32` status code. A panic from your main thread is caught and results in a
//! status code of −1.
//!
//! ### Example
//!
//! ```
//! extern crate px4;
//!
//! use px4::px4_module_main;
//!
//! #[px4_module_main]
//! fn my_module(args: &[&str]) -> i32 {
//!   0
//! }
//! ```
//!
//! ## Logging
//!
//! As soon as your main function is entered, logging is already set up using
//! the standard [`log` crate](https://docs.rs/log/). You can use the standard
//! logging macros such as `info!`, `warn!`, and `error!` to log messages,
//! equivalent to `PX4_INFO` (etc.) in C and C++.
//!
//! Use the `info_raw!` macro to send raw output, equivalent to the
//! `PX4_INFO_RAW` macro in C and C++.
//! Do not use standard output or standard error for this, as the standard
//! streams of the PX4 process are often not the ones connected to the terminal
//! the user is looking at.
//!
//! ### Example
//!
//! ```
//! extern crate log;
//! extern crate px4;
//!
//! use log::{info, warn};
//! use px4::px4_module_main;
//!
//! #[px4_module_main]
//! fn my_module(args: &[&str]) {
//!   info!("Hello World!");
//!   warn!("A warning!");
//!   panic!("Bye!");
//! }
//! ```
//!
//! ## uORB
//!
//! Message definitions can be imported from `.msg` files, and then subscribed
//! to or published. See the [`uorb` module](uorb/index.html) for documentation
//! on how to use the uORB bindings.
//!
//! ### Example
//!
//! ```
//! extern crate log;
//! extern crate px4;
//!
//! use log::info;
//! use px4::{px4_module_main, px4_message};
//! use px4::uorb::{Publish, Subscribe};
//!
//! #[px4_message("example/msg/debug_value.msg")]
//! pub struct debug_value;
//!
//! #[px4_module_main]
//! fn my_module(args: &[&str]) {
//!
//!   let mut publ = debug_value::advertise();
//!   publ.publish(&debug_value { timestamp: 0, value: 1.0, ind: 3 }).unwrap();
//!
//!   let sub = debug_value::subscribe().unwrap();
//!   info!("Latest debug message: {:?}", sub.get().unwrap());
//! }
//! ```

extern crate log;
extern crate px4_macros;

use std::ffi::CStr;
use std::os::raw::c_char;

pub mod uorb;
mod logging;

pub use crate::logging::{log_raw, LogLevel};
pub use px4_macros::{px4_message, px4_module_main};

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
