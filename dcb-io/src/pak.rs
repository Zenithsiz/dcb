//! `.PAK` file parser

// Imports
use std::io;

/// A `.PAK` file
pub struct PakFile {}

impl PakFile {
	/// Deserializes a `.PAK` file from a reader
	pub fn deserialize<R: io::Read>(_reader: R) -> Self {
		Self {}
	}
}
