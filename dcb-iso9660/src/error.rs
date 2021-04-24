//! Errors

// Imports
use super::volume_descriptor;
use dcb_cdrom_xa::reader::{ReadSectorError, SeekSectorError};

/// Error type for [`FilesystemReader::new`](super::FilesystemReader::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to seek to volume descriptor set
	#[error("Unable to seek to volume descriptor set")]
	SeekVolumeDescriptorSet(#[source] SeekSectorError),

	/// Missing primary volume
	#[error("No primary volume found before set terminator")]
	MissingPrimaryVolumeBeforeSetTerminator,

	/// Eof before set terminator
	#[error("Found eof before set terminator")]
	EofBeforeSetTerminator,

	/// Invalid sector before set terminator
	#[error("Invalid sector before set terminator")]
	InvalidSectorBeforeSetTerminator(#[source] ReadSectorError),

	/// Invalid volume descriptor
	#[error("Invalid volume descriptor")]
	InvalidVolumeDescriptor(#[source] volume_descriptor::FromBytesError),

	/// Primary volume must be in form1
	#[error("Primary volume must be in form 1")]
	PrimaryFormatWrongForm,
}
