// Bindings to the C API.

use std::ffi::CStr;

extern "C" {
	pub fn orb_advertise(meta: *const Metadata, data: *const u8) -> usize;
	pub fn orb_advertise_queue(meta: *const Metadata, data: *const u8, queue_size: u32) -> usize;
	pub fn orb_advertise_multi(meta: *const Metadata, data: *const u8, instance: *mut i32, priority: i32) -> usize;
	pub fn orb_advertise_multi_queue(meta: *const Metadata, data: *const u8, instance: *mut i32, priority: i32, queue_size: u32) -> usize;
	pub fn orb_unadvertise(handle: usize) -> i32;
	pub fn orb_publish(meta: *const Metadata, handle: usize, data: *const u8) -> i32;
	pub fn orb_subscribe(meta: *const Metadata) -> i32;
	pub fn orb_subscribe_multi(meta: *const Metadata, instance: u32) -> i32;
	pub fn orb_unsubscribe(handle: i32) -> i32;
	pub fn orb_copy(meta: *const Metadata, handle: i32, buffer: *mut u8) -> i32;
	pub fn orb_check(handle: i32, updated: *mut bool) -> i32;
	pub fn orb_stat(handle: i32, time: *mut u64) -> i32;
	pub fn orb_exists(meta: *const Metadata, instance: i32) -> i32;
	pub fn orb_group_count(meta: *const Metadata) -> i32;
	pub fn orb_priority(handle: i32, priority: *mut i32) -> i32;
	pub fn orb_set_interval(handle: i32, interval: u32) -> i32;
	pub fn orb_get_interval(handle: i32, interval: *mut u32) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Metadata {
	name: *const u8,
	size: u16,
	size_no_padding: u16,
	fields: *const u8,
}

unsafe impl Sync for Metadata {}

impl Metadata {
	// This is used by the #[px4_message] proc_macro.
	#[doc(hidden)]
	pub const fn _unsafe_new(name: *const u8, size: u16, size_no_padding: u16, fields: *const u8) -> Metadata {
		Metadata { name, size, size_no_padding, fields }
	}
	pub fn name(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.name_cstr().to_bytes()) }
	}

	pub fn name_cstr(&self) -> &CStr {
		unsafe { CStr::from_ptr(self.name as *const _) }
	}

	pub fn size(&self) -> u16 {
		self.size
	}
	pub fn size_no_padding(&self) -> u16 {
		self.size_no_padding
	}
	pub fn fields(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.fields_cstr().to_bytes()) }
	}
	pub fn fields_cstr(&self) -> &CStr {
		unsafe { CStr::from_ptr(self.fields as *const _) }
	}
}
