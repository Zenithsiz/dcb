//! [`Try`] types or [`()`] trait

// Imports
use std::ops::{ControlFlow, Try};


/// Trait implemented for all types except [`()`]
pub auto trait NotEmpty {}

impl !NotEmpty for () {}

/// Trait implemented by either `Try<Output = ()>` types or `()`
pub trait TryOrEmpty {
	/// Try type
	type Try: Try<Output = ()>;

	/// Converts this type into the try type
	fn into_try(self) -> Self::Try;
}

impl<T: Try<Output = ()> + NotEmpty> TryOrEmpty for T {
	type Try = T;

	fn into_try(self) -> Self::Try {
		self
	}
}

impl TryOrEmpty for () {
	type Try = ControlFlow<!>;

	fn into_try(self) -> Self::Try {
		ControlFlow::Continue(self)
	}
}
