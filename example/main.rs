extern crate log;
extern crate px4;

use log::{warn, info};
use px4::{info_raw, px4_module_main};

#[px4_module_main]
pub fn main(args: &[&str]) -> i32 {
	info!("Hello World!");

	info_raw!("\n |> \\/ /_|    |> | | /_ |_");
	info_raw!("\n |  /\\   |    |\\ |_|  / |_\n\n");

	info!("Arguments: {:?}", &args[1..]);

	if args.get(1) == Some(&"panic") {
		panic!("A disaster happened!");
	}

	warn!("The end is near!");

	0
}
