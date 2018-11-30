//! Bindings to the uORB messaging system.
//!
//! This part is not yet finished.

mod c;
mod publish;
mod subscribe;

pub use self::c::Metadata;
pub use self::publish::{Publisher, Publish};
pub use self::subscribe::{Subscription, Subscribe};

pub unsafe trait Message {
	fn metadata() -> &'static Metadata;
}
