//! Errors

// Imports
use super::volume_descriptor::{self, DescriptorKind};
use crate::cdrom::SectorError;

/// Error type for [`Filesystem::new`](super::Filesystem::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to read primary volume sector
	#[error("Unable to read primary volume sector")]
	ReadPrimaryVolumeSector(#[source] SectorError),

	/// Unable to parse primary volume
	#[error("Unable to parse primary volume")]
	ParsePrimaryVolume(#[source] volume_descriptor::FromBytesError),

	/// First volume was not the primary volume
	#[error("First volume was not the primary volume, was {_0:?}")]
	FirstVolumeNotPrimary(DescriptorKind),
}
