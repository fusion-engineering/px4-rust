extern crate log;
extern crate px4;

use log::{warn, info};
use px4::{info_raw, px4_module};

pub fn main(args: &[&str]) -> i32 {
	warn!("Hello World!");

	info_raw!("\n |> \\/ /_|    |> | | /_ |_");
	info_raw!("\n |  /\\   |    |\\ |_|  / |_\n\n");

	info!("Arguments: {:?}", &args[1..]);

	if args.len() > 1 && args[1] == "panic" {
		panic!("A disaster happened!");
	}

	0
}

px4_module!(main);
