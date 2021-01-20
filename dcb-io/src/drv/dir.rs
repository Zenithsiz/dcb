//! Directories

// Modules
pub mod entry;
pub mod error;

// Exports
pub use entry::DirEntry;
pub use error::FromBytesError;

/// Directory
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Dir {
	/// All directory entries
	entries: Vec<DirEntry>,
}

impl Dir {
	/// Parses a directory from bytes
	pub fn from_bytes(dir_bytes: &[u8], file_bytes: &[u8]) -> Result<Self, FromBytesError> {
		// Keep reading bytes until we get an entry kind of 0
		// TODO: Maybe error if `bytes` finishes before we find a `0`.
		let entries = dir_bytes
			.array_chunks::<0x20>()
			.map(|entry_bytes| DirEntry::from_bytes(entry_bytes, file_bytes))
			.map_while(|res| match res {
				Err(entry::FromBytesError::InvalidKind(0)) => None,
				res => Some(res),
			})
			.collect::<Result<_, _>>()
			.map_err(FromBytesError::ReadEntry)?;

		Ok(Self { entries })
	}

	/// Returns all the entries in this directory
	#[must_use]
	pub fn entries(&self) -> &[DirEntry] {
		&self.entries
	}
}
