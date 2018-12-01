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

	let mut p = debug_value::advertise();

	assert_eq!(p.is_advertised(), false);

	p.publish(&debug_value {
		timestamp: 123,
		value: 1.0f32,
		ind: 13,
	}).unwrap();

	assert_eq!(p.is_advertised(), true);

	p.publish(&debug_value {
		timestamp: 456,
		value: 2.0f32,
		ind: 37,
	}).unwrap();

	info!("debug_value exists: {}", debug_value::exists(0));
	info!("debug_value group_count: {}", debug_value::group_count());

	let sub = debug_value::subscribe().unwrap();
	info!("Subscribed and read: {:?}", sub.get());

	0
}
