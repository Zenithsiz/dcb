//! Errors

// Imports
use super::entry;
use std::io;

/// Error for [`DirReader::read_entries`](super::DirReader::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum EntriesError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	Seek(#[source] io::Error),
}

/// Error for [`DirReader::write_entries`](super::DirReader::write_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntryError {
	/// Unable to read entry bytes
	#[error("Unable to read entry bytes")]
	ReadEntry(#[source] io::Error),

	/// Unable to parse entry
	#[error("Unable to parse entry")]
	ParseEntry(#[source] entry::FromBytesError),
}

/// Error for [`DirWriter::to_writer`](super::DirWriter::to_writer)
#[derive(Debug, thiserror::Error)]
pub enum WriteEntriesError<E: std::error::Error + 'static> {
	/// Unable to get position
	#[error("Unable to get position")]
	GetPos(#[source] io::Error),

	/// Writer was not at sector star
	#[error("Writer was not at sector start")]
	WriterAtSectorStart,

	/// Writer current sector was past max
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

	/// Unable to seek to entries
	#[error("Unable to seek to entries")]
	SeekToEntries(#[source] io::Error),
	
	/// Unable to write all entries
	#[error("Unable to write entries")]
	WriteEntries(#[source] io::Error),
}
