//! The `.DRV` filesystem

// Modules
pub mod dir;
pub mod error;
pub mod file;

// Exports
pub use dir::Dir;
pub use error::FromBytesError;
pub use file::File;

/// The filesystem
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Filesystem {
	/// Root directory
	root: Dir,
}

impl Filesystem {
	/// Parses a filesystem from bytes
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
		// Read the root directory
		let root = Dir::from_bytes(bytes, bytes).map_err(FromBytesError::RootDir)?;

		Ok(Self { root })
	}

	/// Returns the root directory of this filesystem
	#[must_use]
	pub const fn root(&self) -> &Dir {
		&self.root
	}
}
