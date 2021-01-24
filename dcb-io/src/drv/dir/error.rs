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
pub enum WriteEntriesError {
	/// Unable to get position
	#[error("Unable to get position")]
	GetPos(#[source] io::Error),

	/// Unable to get entry
	#[error("Unable to get entry")]
	GetEntry(#[source] io::Error),

	/// Unable to write entry in directory
	#[error("Unable to write entry in directory")]
	WriteEntryInDir(#[source] io::Error),

	/// Unable to seek to entry
	#[error("Unable to seek to entry")]
	SeekToEntry(#[source] io::Error),

	/// Unable to write file
	#[error("Unable to write file")]
	WriteFile(#[source] io::Error),

	/// Unable to write directory
	#[error("Unable to write directory")]
	WriteDir(#[source] Box<Self>),
}
