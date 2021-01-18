//! Abstraction over the game file.
//!
//! See [`GameFile`] for details

/// Game file reader.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash, Debug)]
pub struct GameFile<R> {
	/// The type to read and write from
	reader: R,
}

// Constructors
impl<R> GameFile<R> {
	/// Creates a new game file from any reader
	pub const fn new(reader: R) -> Self {
		Self { reader }
	}
}
