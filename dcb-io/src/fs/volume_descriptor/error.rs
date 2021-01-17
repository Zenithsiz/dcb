//! Error

// Imports
use super::TypeCode;
use crate::fs::{date_time, dir_record, string};

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Invalid magic
	#[error("Invalid magic {_0:#x?}")]
	InvalidMagic([u8; 5]),

	/// Invalid version
	#[error("Invalid version {_0:#x}")]
	InvalidVersion(u8),

	/// Unable to parse type code
	#[error("Unable to parse type code {_0:?}")]
	TypeCode(TypeCode),

	/// Unable to parse boot record
	#[error("Unable to parse boot record")]
	ParseBootRecord(#[source] ParseBootRecordError),

	/// Unable to parse primary
	#[error("Unable to parse primary")]
	ParsePrimary(#[source] ParsePrimaryError),
}

/// Error type for [`VolumeDescriptor::parse_boot_record`](super::VolumeDescriptor::parse_boot_record)
#[derive(Debug, thiserror::Error)]
pub enum ParseBootRecordError {
	/// Unable to parse system id
	#[error("Unable to parse system id")]
	SystemId(#[source] string::InvalidCharError),

	/// Unable to parse boot id
	#[error("Unable to parse boot id")]
	BootId(#[source] string::InvalidCharError),
}

/// Error type for [`VolumeDescriptor::parse_primary`](super::VolumeDescriptor::parse_primary)
#[derive(Debug, thiserror::Error)]
pub enum ParsePrimaryError {
	/// Unable to parse system id
	#[error("Unable to parse system id")]
	SystemId(#[source] string::InvalidCharError),

	/// Unable to parse volume id
	#[error("Unable to parse volume id")]
	VolumeId(#[source] string::InvalidCharError),

	/// Unable to parse the root dir entry
	#[error("Unable to parse the root dir entry")]
	RootDirEntry(#[source] dir_record::FromBytesError),

	/// Unable to parse volume set id
	#[error("Unable to parse volume set id")]
	VolumeSetId(#[source] string::InvalidCharError),

	/// Unable to parse publisher id
	#[error("Unable to parse publisher id")]
	PublisherId(#[source] string::InvalidCharError),

	/// Unable to parse data preparer id
	#[error("Unable to parse data preparer id")]
	DataPreparerId(#[source] string::InvalidCharError),

	/// Unable to parse application id
	#[error("Unable to parse application id")]
	ApplicationId(#[source] string::InvalidCharError),

	/// Unable to parse copyright file id
	#[error("Unable to parse copyright file id")]
	CopyrightFileId(#[source] string::InvalidCharError),

	/// Unable to parse abstract file id
	#[error("Unable to parse abstract file id")]
	AbstractFileId(#[source] string::InvalidCharError),

	/// Unable to parse bibliographic file id
	#[error("Unable to parse bibliographic file id")]
	BibliographicFileId(#[source] string::InvalidCharError),

	/// Unable to parse volume creation date time
	#[error("Unable to parse volume creation date time")]
	VolumeCreationDateTime(#[source] date_time::FromBytesError),

	/// Unable to parse volume modification date time
	#[error("Unable to parse volume modification date time")]
	VolumeModificationDateTime(#[source] date_time::FromBytesError),

	/// Unable to parse volume expiration date time
	#[error("Unable to parse volume expiration date time")]
	VolumeExpirationDateTime(#[source] date_time::FromBytesError),

	/// Unable to parse volume effective date time
	#[error("Unable to parse volume effective date time")]
	VolumeEffectiveDateTime(#[source] date_time::FromBytesError),
}
