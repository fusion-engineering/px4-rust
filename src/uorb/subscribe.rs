use super::{Message, c};
use std::marker::PhantomData;
use std::mem::uninitialized;

pub struct Subscription<T> {
	handle: i32,
	phantom: PhantomData<Fn() -> T>,
}

pub trait Subscribe {
	fn exists(instance: u32) -> bool;
	fn group_count() -> u32;
	fn subscribe() -> Result<Subscription<Self>, i32>
	where
		Self: Sized;
	fn subscribe_multi(instance: u32) -> Result<Subscription<Self>, i32>
	where
		Self: Sized;
}

impl<T: Message> Subscribe for T {
	fn exists(instance: u32) -> bool {
		unsafe { c::orb_exists(T::metadata(), instance as i32) == 0 }
	}
	fn group_count() -> u32 {
		unsafe { c::orb_group_count(T::metadata()) as u32 }
	}
	fn subscribe() -> Result<Subscription<T>, i32> {
		let handle = unsafe { c::orb_subscribe(T::metadata()) };
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
		let handle = unsafe { c::orb_subscribe_multi(T::metadata(), instance) };
		if handle < 0 {
			Err(handle)
		} else {
			Ok(Subscription {
				handle,
				phantom: PhantomData,
			})
		}
	}
}

impl<T> Drop for Subscription<T> {
	fn drop(&mut self) {
		unsafe { c::orb_unsubscribe(self.handle) };
	}
}

impl<T: Message> Subscription<T> {
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
			let r = c::orb_copy(T::metadata(), self.handle, val as *mut T as *mut u8);
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
			let r = c::orb_check(self.handle, &mut updated);
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
			let r = c::orb_stat(self.handle, &mut time);
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
			let r = c::orb_priority(self.handle, &mut priority);
			if r == 0 {
				Ok(priority)
			} else {
				Err(r)
			}
		}
	}
	pub fn set_interval(&self, interval: u32) -> Result<(), i32> {
		unsafe {
			let r = c::orb_set_interval(self.handle, interval);
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
			let r = c::orb_get_interval(self.handle, &mut interval);
			if r == 0 {
				Ok(interval)
			} else {
				Err(r)
			}
		}
	}
}
