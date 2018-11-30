use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::uninitialized;

extern "C" {
	fn orb_advertise(meta: *const Metadata, data: *const u8) -> usize;
	fn orb_advertise_queue(meta: *const Metadata, data: *const u8, queue_size: u32) -> usize;
	fn orb_advertise_multi(meta: *const Metadata, data: *const u8, instance: *mut i32, priority: i32) -> usize;
	fn orb_advertise_multi_queue(meta: *const Metadata, data: *const u8, instance: *mut i32, priority: i32, queue_size: u32) -> usize;
	fn orb_unadvertise(handle: usize) -> i32;
	fn orb_publish(meta: *const Metadata, handle: usize, data: *const u8) -> i32;
	fn orb_subscribe(meta: *const Metadata) -> i32;
	fn orb_subscribe_multi(meta: *const Metadata, instance: u32) -> i32;
	fn orb_unsubscribe(handle: i32) -> i32;
	fn orb_copy(meta: *const Metadata, handle: i32, buffer: *mut u8) -> i32;
	fn orb_check(handle: i32, updated: *mut bool) -> i32;
	fn orb_stat(handle: i32, time: *mut u64) -> i32;
	fn orb_exists(meta: *const Metadata, instance: i32) -> i32;
	fn orb_group_count(meta: *const Metadata) -> i32;
	fn orb_priority(handle: i32, priority: *mut i32) -> i32;
	fn orb_set_interval(handle: i32, interval: u32) -> i32;
	fn orb_get_interval(handle: i32, interval: *mut u32) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Metadata {
	#[doc(hidden)]
	pub _name: *const u8,
	#[doc(hidden)]
	pub _size: u16,
	#[doc(hidden)]
	pub _size_no_padding: u16,
	#[doc(hidden)]
	pub _fields: *const u8,
}

unsafe impl Sync for Metadata {}

impl Metadata {
	pub fn name_cstr(&self) -> &CStr {
		unsafe { CStr::from_ptr(self._name as *const _) }
	}
	pub fn name(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.name_cstr().to_bytes()) }
	}
	pub fn size(&self) -> u16 {
		self._size
	}
	pub fn size_no_padding(&self) -> u16 {
		self._size_no_padding
	}
	pub fn fields_cstr(&self) -> &CStr {
		unsafe { CStr::from_ptr(self._fields as *const _) }
	}
	pub fn fields(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.fields_cstr().to_bytes()) }
	}
}

#[macro_export]
macro_rules! ORB_ID {
	($name:ident) => {
		ORB_ID!(@ concat!("__orb_", stringify!($name)))
	};
	(@ $name:expr) => {
		unsafe {
			extern "C" {
				#[link_name=$name]
				static metadata: px4::uorb::Metadata;
			}
			&metadata
		}
	};
}

pub unsafe trait MessageMetadata {
	fn metadata() -> &'static Metadata;
}

pub trait Message {
	fn exists(instance: u32) -> bool;
	fn group_count() -> u32;
	fn subscribe() -> Result<Subscription<Self>, i32>
	where
		Self: Sized;
	fn subscribe_multi(instance: u32) -> Result<Subscription<Self>, i32>
	where
		Self: Sized;
	fn advertise(&self) -> Result<Publisher<Self>, ()>
	where
		Self: Sized;
	fn advertise_queue(&self, queue_size: u32) -> Result<Publisher<Self>, ()>
	where
		Self: Sized;
	fn advertise_multi(&self, priority: i32) -> Result<(Publisher<Self>, u32), ()>
	where
		Self: Sized;
	fn advertise_multi_queue(
		&self,
		priority: i32,
		queue_size: u32,
	) -> Result<(Publisher<Self>, u32), ()>
	where
		Self: Sized;
}

impl<T: MessageMetadata> Message for T {
	fn exists(instance: u32) -> bool {
		unsafe { orb_exists(T::metadata(), instance as i32) == 0 }
	}
	fn group_count() -> u32 {
		unsafe { orb_group_count(T::metadata()) as u32 }
	}
	fn subscribe() -> Result<Subscription<T>, i32> {
		let handle = unsafe { orb_subscribe(T::metadata()) };
		if handle < 0 {
			Err(handle)
		} else {
			Ok(Subscription {
				handle,
				phantom: PhantomData,
			})
		}
	}
	fn subscribe_multi(instance: u32) -> Result<Subscription<T>, i32> {
		let handle = unsafe { orb_subscribe_multi(T::metadata(), instance) };
		if handle < 0 {
			Err(handle)
		} else {
			Ok(Subscription {
				handle,
				phantom: PhantomData,
			})
		}
	}
	fn advertise(&self) -> Result<Publisher<T>, ()> {
		let handle = unsafe { orb_advertise(T::metadata(), self as *const T as *const u8) };
		if handle == 0 {
			Err(())
		} else {
			Ok(Publisher {
				handle,
				phantom: PhantomData,
			})
		}
	}
	fn advertise_queue(&self, queue_size: u32) -> Result<Publisher<T>, ()> {
		let handle = unsafe {
			orb_advertise_queue(T::metadata(), self as *const T as *const u8, queue_size)
		};
		if handle == 0 {
			Err(())
		} else {
			Ok(Publisher {
				handle,
				phantom: PhantomData,
			})
		}
	}
	fn advertise_multi(&self, priority: i32) -> Result<(Publisher<T>, u32), ()> {
		let mut instance = 0i32;
		let handle = unsafe {
			orb_advertise_multi(
				T::metadata(),
				self as *const T as *const u8,
				&mut instance,
				priority,
			)
		};
		if handle == 0 {
			Err(())
		} else {
			Ok((
				Publisher {
					handle,
					phantom: PhantomData,
				},
				instance as u32,
			))
		}
	}
	fn advertise_multi_queue(
		&self,
		priority: i32,
		queue_size: u32,
	) -> Result<(Publisher<T>, u32), ()> {
		let mut instance = 0i32;
		let handle = unsafe {
			orb_advertise_multi_queue(
				T::metadata(),
				self as *const T as *const u8,
				&mut instance,
				priority,
				queue_size,
			)
		};
		if handle == 0 {
			Err(())
		} else {
			Ok((
				Publisher {
					handle,
					phantom: PhantomData,
				},
				instance as u32,
			))
		}
	}
}

pub struct Publisher<T> {
	handle: usize,
	phantom: PhantomData<Fn(T)>,
}

impl<T> Drop for Publisher<T> {
	fn drop(&mut self) {
		unsafe { orb_unadvertise(self.handle) };
	}
}

impl<T: MessageMetadata> Publisher<T> {
	pub fn raw_handle(&self) -> usize {
		self.handle
	}
	pub fn publish(&self, value: &T) -> Result<(), i32> {
		let r = unsafe { orb_publish(T::metadata(), self.handle, value as *const T as *const u8) };
		if r == 0 {
			Ok(())
		} else {
			Err(r)
		}
	}
}

pub struct Subscription<T> {
	handle: i32,
	phantom: PhantomData<Fn() -> T>,
}

impl<T> Drop for Subscription<T> {
	fn drop(&mut self) {
		unsafe { orb_unsubscribe(self.handle) };
	}
}

impl<T: MessageMetadata> Subscription<T> {
	pub fn raw_handle(&self) -> i32 {
		self.handle
	}
	pub fn get(&self) -> Result<T, i32> {
		unsafe {
			let mut val = uninitialized::<T>();
			self.copy(&mut val).map(|_| val)
		}
	}
	pub fn copy(&self, val: &mut T) -> Result<(), i32> {
		unsafe {
			let r = orb_copy(T::metadata(), self.handle, val as *mut T as *mut u8);
			if r == 0 {
				Ok(())
			} else {
				Err(r)
			}
		}
	}
	pub fn check(&self) -> Result<bool, i32> {
		unsafe {
			let mut updated = false;
			let r = orb_check(self.handle, &mut updated);
			if r == 0 {
				Ok(updated)
			} else {
				Err(r)
			}
		}
	}
	pub fn stat(&self) -> Result<u64, i32> {
		unsafe {
			let mut time = 0u64;
			let r = orb_stat(self.handle, &mut time);
			if r == 0 {
				Ok(time)
			} else {
				Err(r)
			}
		}
	}
	pub fn get_priority(&self) -> Result<i32, i32> {
		unsafe {
			let mut priority = 0i32;
			let r = orb_priority(self.handle, &mut priority);
			if r == 0 {
				Ok(priority)
			} else {
				Err(r)
			}
		}
	}
	pub fn set_interval(&self, interval: u32) -> Result<(), i32> {
		unsafe {
			let r = orb_set_interval(self.handle, interval);
			if r == 0 {
				Ok(())
			} else {
				Err(r)
			}
		}
	}
	pub fn get_interval(&self) -> Result<u32, i32> {
		unsafe {
			let mut interval = 0u32;
			let r = orb_get_interval(self.handle, &mut interval);
			if r == 0 {
				Ok(interval)
			} else {
				Err(r)
			}
		}
	}
}
