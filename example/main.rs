extern crate log;
extern crate px4;

use log::info;
use px4::{info_raw, px4_message, px4_module_main};

use px4::uorb::{Publish, Subscribe};

#[px4_message("example/msg/debug_value.msg")]
pub struct debug_value;

#[px4_module_main]
fn main(args: &[&str]) -> i32 {
	info!("Hello World!");

	info_raw!("\n |> \\/ /_|    |> | | /_ |_");
	info_raw!("\n |  /\\   |    |\\ |_|  / |_\n\n");

	info!("Arguments: {:?}", &args[1..]);

	info!("Publishing data...");

	let mut d = debug_value {
		timestamp: 123,
		value: 1.0f32,
		ind: 13,
	};
	let p = d.advertise().unwrap();

	d.timestamp = 456;
	p.publish(&d).unwrap();

	info!("debug_value exists: {}", debug_value::exists(0));
	info!("debug_value group_count: {}", debug_value::group_count());

	let sub = debug_value::subscribe().unwrap();
	info!("Subscribed and read: {:?}", sub.get());

	0
}
