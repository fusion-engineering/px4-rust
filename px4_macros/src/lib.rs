#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

mod message;
mod module_main;

#[proc_macro_attribute]
pub fn px4_message(args: TokenStream, input: TokenStream) -> TokenStream {
	message::px4_message(args, input)
}

#[proc_macro_attribute]
pub fn px4_module_main(args: TokenStream, input: TokenStream) -> TokenStream {
	module_main::px4_module_main(args, input)
}
