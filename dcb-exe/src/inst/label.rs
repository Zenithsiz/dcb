//! Labels

// Imports
use std::{
	borrow::Borrow,
	ops::{Deref, DerefMut},
};

/// A label
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct Label(String);

impl Label {
	/// Creates a new label from a string
	#[must_use]
	pub const fn new(name: String) -> Self {
		Self(name)
	}

	/// Returns if this label is global
	#[must_use]
	pub fn is_global(&self) -> bool {
		// We're global if we don't have a `.` within us
		self.find('.').is_none()
	}

	/// Returns this label as a string
	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl Deref for Label {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Label {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Borrow<String> for Label {
	fn borrow(&self) -> &String {
		&self.0
	}
}

impl Borrow<str> for Label {
	fn borrow(&self) -> &str {
		&self.0
	}
}
