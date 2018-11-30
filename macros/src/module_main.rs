use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn px4_module_main(args: TokenStream, input: TokenStream) -> TokenStream {
	if args.to_string() != "" {
		panic!("px4_module_main does not take any arguments");
	}
	let fndef = parse_macro_input!(input as syn::ItemFn);
	let name = &fndef.ident;
	let expanded = quote! {
		#fndef
		#[no_mangle]
		pub extern "C" fn px4_module_main(argc: u32, argv: *mut *mut u8) -> i32 {
			px4::_run(concat!(module_path!(), "\0").as_bytes(), argc, argv, #name)
		}
	};
	expanded.into()
}
