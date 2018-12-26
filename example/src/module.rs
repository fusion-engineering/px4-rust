use log::{info, warn};
use px4::uorb::{Publish, Subscribe};
use px4::{info_raw, px4_message, px4_module_main};
use structopt::StructOpt;

#[px4_message("msg/debug_value.msg")]
pub struct debug_value;

#[derive(StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::DisableVersion"))]
struct Options {
	/// Who to say hello to.
	#[structopt(long, default_value = "World")]
	name: String,

	/// Panic right away, before doing anything.
	#[structopt(long)]
	panic: bool,
}

#[px4_module_main]
fn main(args: &[&str]) -> Result<(), ()> {
	let args = Options::from_iter_safe(args).map_err(|e| info_raw!("{}\n", e))?;

	if args.panic {
		panic!("Oh no, panic!");
	}

	// Logging:

	warn!("Hello {}!", args.name);

	info_raw!("\n |) \\/ /_|    |) | | /_ |_");
	info_raw!("\n |  /\\   |    |\\ |_|  / |_\n\n");

	// Publishing:

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

	// Subscribing:

	info!("debug_value exists: {}", debug_value::exists(0));
	info!("debug_value group_count: {}", debug_value::group_count());

	let sub = debug_value::subscribe().unwrap();

	info!("Subscribed and read: {:?}", sub.get());

	Ok(())
}
