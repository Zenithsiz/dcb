//! Paths
//!
//! See the [`Path`] type for more details.

// Modules
#[cfg(test)]
mod test;

// Imports
use ascii::{AsciiChar, AsciiStr, AsciiString};
use ref_cast::RefCast;
use std::{fmt, iter::FusedIterator, ops};

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
	pub const fn components(&self) -> Components {
		Components::new(self)
	}

	/// Converts this path into a [`PathBuf`]
	#[must_use]
	pub fn to_path_buf(&self) -> PathBuf {
		PathBuf(self.0.to_ascii_string())
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

impl<I> ops::Index<I> for Path
where
	AsciiStr: ops::Index<I, Output = AsciiStr>,
{
	type Output = Self;

	fn index(&self, index: I) -> &Self::Output {
		Self::new(&self.as_ascii()[index])
	}
}

impl fmt::Display for Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_ascii())
	}
}

/// Path Buffer
#[derive(Clone, Debug)]
pub struct PathBuf(AsciiString);

impl ops::Deref for PathBuf {
	type Target = Path;

	fn deref(&self) -> &Self::Target {
		Path::new(&self.0)
	}
}

impl PartialEq for PathBuf {
	fn eq(&self, other: &Self) -> bool {
		**self == **other
	}
}

impl fmt::Display for PathBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", &**self)
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
	pub(self) const fn new(path: &'a Path) -> Self {
		// Trim all trailing `\`
		Self { path }
	}

	/// Returns the remaining path
	#[must_use]
	pub const fn remaining(&self) -> &'a Path {
		self.path
	}
}

impl<'a> FusedIterator for Components<'a> {}

impl<'a> Iterator for Components<'a> {
	type Item = Component<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		// Read until the next `\\` or eof
		let (cmpt, rest) = match self.path.0.chars().position(|ch| ch == AsciiChar::BackSlash) {
			// If we found it first, emit a root component
			Some(0) => (Component::Root, self.path),

			// Else it's a normal component
			// Note: We handle `.` and `..` below
			Some(idx) => (Component::Normal(self.path[..idx].as_ascii()), &self.path[idx..]),

			// If we didn't find `\\`, but we're not empty, return the remaining path
			None if !self.path.is_empty() => (Component::Normal(self.path.as_ascii()), Path::empty()),

			// Else we're done
			None => return None,
		};

		// Trim all remaining leading `\\`s
		let rest = rest.trim_leading();

		// If the component is a normal `.` or `..`, change it
		let cmpt = match cmpt {
			Component::Normal(path) if path == "." => Component::CurDir,
			Component::Normal(path) if path == ".." => Component::ParentDir,
			_ => cmpt,
		};

		self.path = rest;
		Some(cmpt)
	}
}

/// Component
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Component<'a> {
	/// Root, `\\` at the start of the path
	Root,

	/// Cur dir, `.`
	CurDir,

	/// Parent dir, `..`
	ParentDir,

	/// Normal
	Normal(&'a AsciiStr),
}
