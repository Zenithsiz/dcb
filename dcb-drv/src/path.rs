//! Paths

// Imports
use ascii::{AsciiChar, AsciiStr};
use ref_cast::RefCast;

/// A path
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

	/// Returns an iterator over all components of this path
	#[must_use]
	pub const fn components(&self) -> Components<'_> {
		Components { path: self, idx: 0 }
	}

	/// Returns the common ancestor between this path and another, as well as the rest
	#[must_use]
	pub fn common_ancestor<'a>(&'a self, other: &'a Self) -> (&'a Self, &'a Self, &'a Self) {
		// While both components are equal, continue them
		let mut lhs = self.components();
		let mut rhs = other.components();

		loop {
			match (lhs.next(), rhs.next()) {
				// If the paths are equal, continue
				// Note: Indexes may not be equal due to normalization
				(Some((_, lhs)), Some((_, rhs))) if lhs == rhs => continue,

				// If they're not, return the common part
				(Some((lhs_idx, _)), Some((rhs_idx, _))) => {
					break (
						Self::new(&self.0[..lhs_idx]),
						Self::new(&self.0[lhs_idx..]),
						Self::new(&other.0[rhs_idx..]),
					)
				},

				(Some((lhs_idx, _)), _) => {
					break (
						Self::new(&self.0[..lhs_idx]),
						Self::new(&self.0[lhs_idx..]),
						Self::empty(),
					)
				},
				(_, Some((rhs_idx, _))) => {
					break (
						Self::new(&other.0[..rhs_idx]),
						Self::empty(),
						Self::new(&other.0[rhs_idx..]),
					)
				},

				// Else they're equal, return ourself
				(None, None) => break (self, Self::empty(), Self::empty()),
			}
		}
	}

	/// Splits this path at it's first component
	#[must_use]
	pub fn split_first(&self) -> Option<(&Self, &Self)> {
		let mut components = self.components();
		let (_, first) = components.next()?;
		Some((first, components.path))
	}

	/// Splits this path at it's last component
	#[must_use]
	pub fn split_last(&self) -> Option<(&Self, &Self)> {
		let (idx, _) = self.components().last()?;
		Some((Self::new(&self.0[..idx]), Self::new(&self.0[idx..])))
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
				(Some((_, lhs)), Some((_, rhs))) if lhs == rhs => continue,

				// If we got to the end, return true
				(None, None) => break true,

				// If they're not, return false
				_ => break false,
			}
		}
	}
}

/// Components of a path
pub struct Components<'a> {
	/// Remaining path
	path: &'a Path,

	/// Current index
	idx: usize,
}

impl<'a> Iterator for Components<'a> {
	type Item = (usize, &'a Path);

	fn next(&mut self) -> Option<Self::Item> {
		// Trim any '\' at the start
		let start_idx = self
			.path
			.0
			.chars()
			.position(|ch| ch != AsciiChar::BackSlash)
			.unwrap_or(0);
		self.path = Path::new(&self.path.0[start_idx..]);
		self.idx += start_idx;

		// Then split at the first `\` we find
		let (path, rest) = match self.path.0.chars().position(|ch| ch == AsciiChar::BackSlash) {
			Some(idx) => (Path::new(&self.path.0[..idx]), Path::new(&self.path.0[idx..])),
			None if !self.path.is_empty() => (self.path, Path::empty()),
			None => return None,
		};

		let idx = self.idx;

		self.path = rest;
		self.idx += path.len();

		Some((idx, path))
	}
}
