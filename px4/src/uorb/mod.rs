//! Bindings to the uORB messaging system.
//!
//! ## Message definitions
//!
//! This crate provides a way to import a message definition from a `.msg`
//! file:
//!
//! ```ignore
//! use px4::px4_message;
//!
//! #[px4_message("msg/foo.msg")] pub struct foo;
//! ```
//!
//! This will read `msg/foo.msg`, relative to the root of the crate (where your
//! Cargo.toml is), parse its contents, and generate the equivalent Rust
//! struct. In addition, [`Message`](trait.Message.html), `Clone` and `Debug`
//! are derived automatically.
//!
//! ## Subscribing
//!
//! Subscribing is done through the [`Subscribe` trait](trait.Subscribe.html),
//! which is automatically implemented for all messages.
//!
//! ```ignore
//! use px4::uorb::Subscribe;
//!
//! let sub = foo::subscribe().unwrap();
//!
//! info!("Latest foo: {:?}", sub.get().unwrap());
//! ```
//!
//! ## Publishing
//!
//! Publishing is done through the [`Publish` trait](trait.Publish.html),
//! which is automatically implemented for all messages.
//!
//! ```ignore
//! use px4::uorb::Publish;
//!
//! let mut publ = foo::advertise();
//!
//! publ.publish(&foo { timestamp: 123, a: 4, b: 5 }).unwrap();
//! ```

mod c;
mod publish;
mod subscribe;

pub use self::c::{priority, Metadata};
pub use self::publish::{Publish, Publisher};
pub use self::subscribe::{Subscribe, Subscription};

/// A message which can be published and/or subscribed to.
///
/// This trait is automatically implemented for all messages imported using
/// `#[px4_message]`.
pub unsafe trait Message {
	/// Get the metadata of this type of message.
	fn metadata() -> &'static Metadata;
}
