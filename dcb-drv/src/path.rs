//! Paths
//!
//! See the [`Path`] type for more details.

// Imports
use ascii::{AsciiChar, AsciiStr};
use ref_cast::RefCast;

/// A path
///
/// Paths are separated by a backslash, `\`.
#[derive(Debug)]
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct Path(AsciiStr);

impl Path {
	/// Creates a new path
	#[must_use]
	pub fn new(path: &AsciiStr) -> &Self {
		Self::ref_cast(path)
	}

	/// Returns an empty path
	#[must_use]
	pub fn empty() -> &'static Self {
		Path::new(AsciiStr::from_ascii("").expect("Empty string wasn't valid ascii"))
	}

	/// Returns this path as a string
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	/// Returns this path as an ascii string
	#[must_use]
	pub const fn as_ascii(&self) -> &AsciiStr {
		&self.0
	}

	/// Returns this path's length
	#[must_use]
	pub fn len(&self) -> usize {
		self.as_str().len()
	}

	/// Returns if this path is empty
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Trims leading slashes
	#[must_use]
	pub fn trim_leading(mut self: &Self) -> &Self {
		while let [AsciiChar::BackSlash, path @ ..] = self.0.as_slice() {
			self = Path::new(path.into());
		}

		self
	}

	/// Trims trailing slashes
	#[must_use]
	pub fn trim_trailing(mut self: &Self) -> &Self {
		while let [path @ .., AsciiChar::BackSlash] = self.0.as_slice() {
			self = Path::new(path.into());
		}

		self
	}

	/// Returns an iterator over all components of this path
	#[must_use]
	pub fn components(&self) -> Components {
		Components::new(self)
	}

	/// Splits this path at it's first component
	#[must_use]
	pub fn split_first(&self) -> Option<(&AsciiStr, &Self)> {
		let mut components = self.components();
		let first = components.next()?;
		Some((first.as_ascii(), components.path))
	}

	/// Splits this path at it's last component
	#[must_use]
	pub fn split_last(&self) -> Option<(&Self, &AsciiStr)> {
		// Get the last component
		let (idx, last) = self.components().enumerate().last()?;

		// If it was the start component, return
		if idx == 0 {
			return None;
		}

		// Else separate them
		let start = &self.0[..(self.len() - last.len())];
		Some((Self::new(start), last.as_ascii()))
	}
}

impl PartialEq for Path {
	fn eq(&self, other: &Self) -> bool {
		// Compare the components
		let mut lhs = self.components();
		let mut rhs = other.components();

		loop {
			match (lhs.next(), rhs.next()) {
				// If the paths are equal, continue
				// Note: Indexes may not be equal due to normalization
				(Some(lhs), Some(rhs)) if lhs == rhs => continue,

				// If we got to the end, return true
				(None, None) => break true,

				// If they're not, return false
				_ => break false,
			}
		}
	}
}

/// Components of a path
#[derive(PartialEq, Clone, Debug)]
pub struct Components<'a> {
	/// Remaining path
	path: &'a Path,
}

impl<'a> Components<'a> {
	/// Creates new components
	pub(self) fn new(path: &'a Path) -> Self {
		// Trim all trailing `\`
		Self {
			path: path.trim_trailing(),
		}
	}
}

impl<'a> Iterator for Components<'a> {
	type Item = &'a Path;

	fn next(&mut self) -> Option<Self::Item> {
		// Trim all leading `\`
		self.path = self.path.trim_leading();

		// Then split at the first `\` we find
		let (path, rest) = match self.path.0.chars().position(|ch| ch == AsciiChar::BackSlash) {
			Some(idx) => (Path::new(&self.path.0[..idx]), Path::new(&self.path.0[idx..])),
			None if !self.path.is_empty() => (self.path, Path::empty()),
			None => return None,
		};

		self.path = rest;
		Some(path)
	}
}
