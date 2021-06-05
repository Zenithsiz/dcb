//! Void

/// Void a value explicitly
pub trait Void: Sized {
	/// Void this value
	fn void(self) {}
}

impl<T> Void for T {}
