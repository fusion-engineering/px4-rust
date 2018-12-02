use super::{c, priority, Message};
use std::marker::PhantomData;
use std::ptr::null_mut;

enum PublisherState {
	Unadvertised {
		priority: Option<i32>,
		queue_size: u32,
	},
	Advertised {
		handle: usize,
		instance: u32,
	},
}

/// A publisher of [`Message`](trait.Message.html)s.
///
/// Create one using one of the
/// [`advertise` functions](trait.Publish.html).
///
/// [`Drop`](struct.Publisher.html#impl-Drop)ping the publisher will
/// unadvertise the message.
pub struct Publisher<T> {
	state: PublisherState,
	phantom: PhantomData<fn(T)>,
}

impl<T: Message> Publisher<T> {
	/// Publish a message.
	///
	/// The first time this funciton is called, it will call advertise the message.
	pub fn publish(&mut self, value: &T) -> Result<(), i32> {
		assert_eq!(std::mem::size_of::<T>(), T::metadata().size() as usize);
		let value_ptr = value as *const T as *const u8;
		match self.state {
			PublisherState::Unadvertised {
				priority,
				queue_size,
			} => {
				let mut instance = 0i32;
				let handle = unsafe {
					c::orb_advertise_multi_queue(
						T::metadata(),
						value_ptr,
						if priority.is_some() {
							&mut instance
						} else {
							null_mut()
						},
						priority.unwrap_or(priority::DEFAULT),
						queue_size,
					)
				};
				if handle == 0 {
					Err(0)
				} else {
					self.state = PublisherState::Advertised {
						handle,
						instance: if priority.is_some() {
							instance as u32
						} else {
							u32::max_value()
						},
					};
					Ok(())
				}
			}
			PublisherState::Advertised { handle, .. } => {
				let r = unsafe { c::orb_publish(T::metadata(), handle, value_ptr) };
				if r == 0 {
					Ok(())
				} else {
					Err(r)
				}
			}
		}
	}
}

impl<T> Publisher<T> {
	fn new(priority: Option<i32>, queue_size: u32) -> Self {
		Publisher {
			state: PublisherState::Unadvertised {
				priority,
				queue_size,
			},
			phantom: PhantomData,
		}
	}

	/// Check whether the message is already advertised.
	///
	/// Will be true after the first call to
	/// [`publish`](struct.Publisher.html#method.publish).
	pub fn is_advertised(&self) -> bool {
		match self.state {
			PublisherState::Advertised { .. } => true,
			_ => false,
		}
	}

	/// Get the instance number of the published message.
	///
	/// Only available after the first call to
	/// [`publish`](struct.Publisher.html#method.publish),
	/// for publishers created through
	/// [`advertise_multi`](trait.Publish.html#tymethod.advertise_multi) or
	/// [`advertise_multi_queue`](trait.Publish.html#tymethod.advertise_multi_queue).
	pub fn instance(&self) -> Option<u32> {
		match self.state {
			PublisherState::Advertised { instance, .. } if instance != u32::max_value() => {
				Some(instance)
			}
			_ => None,
		}
	}

	/// Get the raw `orb_advert_t`.
	///
	/// Will return 0 before the first call to
	/// [`publish`](struct.Publisher.html#method.publish).
	pub fn raw_handle(&self) -> usize {
		match self.state {
			PublisherState::Advertised { handle, .. } => handle,
			_ => 0,
		}
	}
}

impl<T> Drop for Publisher<T> {
	fn drop(&mut self) {
		let handle = self.raw_handle();
		if handle != 0 {
			unsafe { c::orb_unadvertise(handle) };
		}
	}
}

/// Use one of the functions below to create a [`Publisher`](struct.Publisher.html).
///
/// They are automatically implemented on all [`Message`](trait.Message.html)s.
///
/// The functions are lazy: The messages aren't advertised directly, but only
/// on the first call to [`publish`](struct.Publisher.html#method.publish).
pub trait Publish {
	fn advertise() -> Publisher<Self>
	where
		Self: Sized;
	fn advertise_queue(queue_size: u32) -> Publisher<Self>
	where
		Self: Sized;
	fn advertise_multi(priority: i32) -> Publisher<Self>
	where
		Self: Sized;
	fn advertise_multi_queue(priority: i32, queue_size: u32) -> Publisher<Self>
	where
		Self: Sized;
}

impl<T: Message> Publish for T {
	fn advertise() -> Publisher<T> {
		Publisher::new(None, 1)
	}
	fn advertise_queue(queue_size: u32) -> Publisher<T> {
		Publisher::new(None, queue_size)
	}
	fn advertise_multi(priority: i32) -> Publisher<T> {
		Publisher::new(Some(priority), 1)
	}
	fn advertise_multi_queue(priority: i32, queue_size: u32) -> Publisher<T> {
		Publisher::new(Some(priority), queue_size)
	}
}
