//! Errors

// Imports
use std::io;

/// Error for [`DirWriter::to_writer`](super::DirWriter::to_writer)
#[derive(Debug, thiserror::Error)]
pub enum WriteEntriesError<E: std::error::Error + 'static> {
	/// Unable to get current sector
	#[error("Unable to get current sector")]
	GetSectorPos(#[source] io::Error),

	/// Writer was not at the start of a sector
	#[error("Writer was not at the start of a sector")]
	WriterNotAtSectorStart,

	/// Writer current sector was past maximum
	#[error("Writer current sector was past `u32::MAX`")]
	WriterSectorPastMax,

	/// Unable to get entry
	#[error("Unable to get entry")]
	GetEntry(#[source] E),

	/// Unable to seek to entry
	#[error("Unable to seek to entry")]
	SeekToEntry(#[source] io::Error),

	/// Unable to write file
	#[error("Unable to write file")]
	WriteFile(#[source] io::Error),

	/// Unable to write directory
	#[error("Unable to write directory")]
	WriteDir(#[source] Box<Self>),

	/// Unable to seek to directory entries
	#[error("Unable to seek to directory entries")]
	SeekEntries(#[source] io::Error),

	/// Unable to write all directory entries
	#[error("Unable to write directory entries")]
	WriteEntries(#[source] io::Error),
}
