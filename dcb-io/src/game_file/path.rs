//! Game file paths

// Imports
use ascii::{AsciiChar, AsciiStr};
use ref_cast::RefCast;

/// Game file path
#[derive(PartialEq, Debug)]
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct Path(AsciiStr);

impl Path {
	/// Creates a new path
	#[must_use]
	pub fn new(path: &AsciiStr) -> &Self {
		Self::ref_cast(path)
	}

	/// Returns this path as a string
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	/// Returns this path's drive and remaining path, if any
	#[must_use]
	pub fn drive(&self) -> Option<(AsciiChar, &Self)> {
		match self.0.as_slice() {
			[drive, AsciiChar::Colon, AsciiChar::BackSlash, rest @ ..] if drive.is_alphabetic() => {
				Some((*drive, Self::new(rest.into())))
			},
			_ => None,
		}
	}
}
