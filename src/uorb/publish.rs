use super::{Message, c};
use std::marker::PhantomData;

pub struct Publisher<T> {
	handle: usize,
	phantom: PhantomData<Fn(T)>,
}

pub trait Publish {
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

impl<T: Message> Publish for T {
	fn advertise(&self) -> Result<Publisher<T>, ()> {
		let handle = unsafe { c::orb_advertise(T::metadata(), self as *const T as *const u8) };
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
			c::orb_advertise_queue(T::metadata(), self as *const T as *const u8, queue_size)
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
			c::orb_advertise_multi(
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
			c::orb_advertise_multi_queue(
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

impl<T> Drop for Publisher<T> {
	fn drop(&mut self) {
		unsafe { c::orb_unadvertise(self.handle) };
	}
}

impl<T: Message> Publisher<T> {
	pub fn raw_handle(&self) -> usize {
		self.handle
	}
	pub fn publish(&self, value: &T) -> Result<(), i32> {
		let r =
			unsafe { c::orb_publish(T::metadata(), self.handle, value as *const T as *const u8) };
		if r == 0 {
			Ok(())
		} else {
			Err(r)
		}
	}
}
