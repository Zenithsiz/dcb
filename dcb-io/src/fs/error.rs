//! Errors

// Imports
use super::volume_descriptor;
use crate::game_file::SectorError;

/// Error type for [`Filesystem::new`](super::Filesystem::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to read primary volume sector
	#[error("Unable to read primary volume sector")]
	ReadPrimaryVolumeSector(#[source] SectorError),

	/// Unable to parse primary volume
	#[error("Unable to parse primary volume")]
	ParsePrimaryVolume(#[source] volume_descriptor::FromBytesError),
}
