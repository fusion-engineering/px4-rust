use memoffset::span_of;
use px4::px4_message;
use px4::uorb::Message;
use std::mem::size_of;

#[px4_message("tests/message_macro/test.msg")]
struct test_message;

#[test]
fn generated_message() {

	// The generated metadata:

	let m = test_message::metadata();
	assert_eq!(m.name(), "test_message");
	assert_eq!(m.fields(), "\
		uint64_t value;\
		int16_t[12] array;\
		bool[2] array2;\
		int8_t value2;\
		char ch;\
		uint8_t[4] _padding0;\
	");
	assert_eq!(m.size(), 40);
	assert_eq!(m.size_no_padding(), 36);

	// The fields and their types in the generated struct:

	let _ = test_message {
		array: [1i16, -2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
		array2: [true, false],
		value: 123u64,
		value2: 99i8,
		ch: b'a',
	};

	// The exact layout of the generated struct:

	assert_eq!(size_of::<test_message>(), 40);
	assert_eq!(span_of!(test_message, value), 0..8);
	assert_eq!(span_of!(test_message, array), 8..32);
	assert_eq!(span_of!(test_message, array2), 32..34);
	assert_eq!(span_of!(test_message, value2), 34..35);
	assert_eq!(span_of!(test_message, ch), 35..36);
}
