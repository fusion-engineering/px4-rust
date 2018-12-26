use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use syn::parse_macro_input;

pub fn px4_message(args: TokenStream, input: TokenStream) -> TokenStream {
	let arg = parse_macro_input!(args as syn::LitStr).value();

	// Open .msg file

	let path = if let Some(root) = std::env::var_os("CARGO_MANIFEST_DIR") {
		Path::new(&root).join(&arg)
	} else {
		arg.into()
	};
	let file = File::open(&path).unwrap_or_else(|e| {
		panic!("Unable to open {:?}: {}", path, e);
	});
	let file = BufReader::new(file);

	// Verify that the struct looks like `[pub] struct name;`

	let input = parse_macro_input!(input as syn::DeriveInput);
	let name = input.ident;
	let is_unit_struct = match input.data {
		syn::Data::Struct(s) => match s.fields {
			syn::Fields::Unit => true,
			_ => false,
		},
		_ => false,
	};
	if !is_unit_struct || input.generics.lt_token.is_some() {
		panic!("Expected `struct {};`", name);
	}

	// Read the .msg file line by line, collecting all the struct members.

	let mut members = Vec::new();

	for (line_num, line) in file.lines().enumerate() {
		// Parse the lines, throwing away comments and empty lines, splitting them in type and name.

		let mut line = line.unwrap_or_else(|e| {
			panic!("Unable to read from {:?}: {}", path, e);
		});
		if let Some(comment_start) = line.find('#') {
			line.truncate(comment_start);
		}
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		let mut words = line.split_whitespace();
		let mut type_ = words.next().unwrap();
		let name = words.next().unwrap_or_else(|| {
			panic!("Missing name on line {} in {:?}", line_num + 1, path);
		});
		if words.next().is_some() {
			panic!(
				"Garbage after end of line on line {} in {:?}",
				line_num + 1,
				path
			);
		}

		// Parse array types.

		let array_len = type_.find('[').map(|open_brace| {
			if !type_.ends_with(']') {
				panic!("Missing `]` on line {} in {:?}", line_num + 1, path);
			}
			let braced_part = &type_[open_brace + 1..type_.len() - 1];
			type_ = &type_[..open_brace];
			braced_part.parse::<usize>().unwrap_or_else(|_| {
				panic!(
					"Invalid array length on line {} in {:?}",
					line_num + 1,
					path
				);
			})
		});

		// Look up the type's width, Rust type, and C type.

		let (width, mut rust, c) = match type_ {
			"uint64"         => (8, quote! { u64  }, "uint64_t"),
			"uint32"         => (4, quote! { u32  }, "uint32_t"),
			"uint16"         => (2, quote! { u16  }, "uint16_t"),
			"uint8" | "byte" => (1, quote! { u8   }, "uint8_t"),
			"int64"          => (8, quote! { i64  }, "int64_t"),
			"int32"          => (4, quote! { i32  }, "int32_t"),
			"int16"          => (2, quote! { i16  }, "int16_t"),
			"int8"           => (1, quote! { i8   }, "int8_t"),
			"float64"        => (8, quote! { f64  }, "double"),
			"float32"        => (4, quote! { f32  }, "float"),
			"char"           => (1, quote! { u8   }, "char"),
			"bool"           => (1, quote! { bool }, "bool"),
			_ => {
				panic!(
					"Unknown type `{}` on line {} in {:?}",
					type_,
					line_num + 1,
					path
				);
			}
		};
		let mut c = c.to_string();
		if let Some(n) = array_len {
			rust = quote! { [#rust; #n] };
			c = format!("{}[{}]", c, n);
		}

		// Add it to the list.

		let name = syn::Ident::new(name, Span::call_site());
		let size = array_len.unwrap_or(1) * width;
		members.push((name, width, rust, c, size));
	}

	// Sort the members by alignment, biggest first.

	members.sort_by(|a, b| b.1.cmp(&a.1));

	// Compute the total size and add any padding.

	let mut fields = String::new();
	let mut size = 0;
	let mut pad_num = 0;
	for m in &members {
		add_padding(&mut fields, &mut pad_num, &mut size, m.1);
		write!(&mut fields, "{} {};", m.3, m.0).unwrap();
		size += m.4;
	}
	let size_no_padding = size;
	add_padding(&mut fields, &mut pad_num, &mut size, 8);
	fields.push('\0');

	if size > 0xFFFF {
		panic!("Message size too big");
	}

	// Generate the Rust code.

	let size = size as u16;
	let size_no_padding = size_no_padding as u16;
	let path = path.to_str().unwrap();
	let vis = input.vis;
	let attrs = input.attrs;
	let mems = members.iter().map(|m| {
		let n = &m.0;
		let t = &m.2;
		quote! { #n: #t }
	});
	let name_str = format!("{}\0", name);

	let expanded = quote! {
		#[repr(C)]
		#[repr(align(8))]
		#[derive(Clone, Debug)]
		#(#attrs)*
		#vis struct #name {
			#(#mems),*
		}
		unsafe impl px4::uorb::Message for #name {
			fn metadata() -> &'static px4::uorb::Metadata {
				let _ = include_bytes!(#path); // This causes the file to be recompiled if the .msg-file is changed.
				static M: px4::uorb::Metadata = px4::uorb::Metadata::_unsafe_new(
					#name_str.as_ptr(),
					#size,
					#size_no_padding,
					#fields.as_ptr(),
				);
				&M
			}
		}
	};

	expanded.into()
}

fn add_padding(fields: &mut String, pad_num: &mut usize, size: &mut usize, alignment: usize) {
	let misalignment = *size % alignment;
	if misalignment != 0 {
		let pad = alignment - misalignment;
		write!(fields, "uint8_t[{}] _padding{};", pad, *pad_num).unwrap();
		*size += pad;
		*pad_num += 1;
	}
}
