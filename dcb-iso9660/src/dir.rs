//! A directory

// Imports
use crate::DirEntry;

/// A directory
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Dir {
	/// All entries in the directory
	entries: Vec<DirEntry>,
}

impl Dir {
	/// Creates a new directory from it's entries
	pub fn new(entries: impl IntoIterator<Item = DirEntry>) -> Self {
		Self {
			entries: entries.into_iter().collect(),
		}
	}

	/// Returns all entries in this directory
	#[must_use]
	pub fn entries(&self) -> &[DirEntry] {
		&self.entries
	}

	/// Finds an entry in this directory
	#[must_use]
	pub fn find<'a>(&'a self, name: &str) -> Option<&'a DirEntry> {
		// TODO: Avoid allocation
		self.entries.iter().find(|entry| entry.name.to_string() == name)
	}
}
