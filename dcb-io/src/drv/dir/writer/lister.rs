//! Directory writer lister

// Imports
use crate::drv::DirEntryWriter;
use std::io;

/// Directory lister
///
/// This trait serves to provide an interface for the caller
/// to list all files and directories they want to write to
/// the drv file without requiring them all to be read before
/// hand.
///
/// It is similar to a visitor, but with the roles reversed,
/// it is the writer that visits the entries provided by the
/// user by means of the [`IntoIterator`] implementation.
///
/// It is required to know the size before-hand to know how
/// much space to allocate for each directory. It would be
/// possible to write the children first and then the directory
/// to avoid this problem, but the root directory would still
/// need to report it's size, and by putting the directories first
/// readers don't have to seek across the file to read the full
/// directory tree as much.
/// This is, of course, provided the implementor supplies directories
/// before the files, else they will not be placed at the start.
pub trait DirWriterLister: Sized
where
	Self: IntoIterator<Item = Result<DirEntryWriter<Self>, <Self as DirWriterLister>::Error>>,
{
	/// Type used to read all files in this directory tree
	type FileReader: io::Read;

	/// Error type for each entry
	type Error: std::error::Error + 'static;

	/// Returns the number of entries in this lister
	fn entries_len(&self) -> u32;
}
