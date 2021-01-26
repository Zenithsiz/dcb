//! Errors

// Imports
use crate::sector;

/// Error type for [`CdRom::write_sector`](super::CdRom::write_sector)
#[derive(Debug, thiserror::Error)]
pub enum WriteSectorError {
	/// Unable to serialize sector to bytes
	#[error("Unable to serialize sector to bytes")]
	ToBytes(#[source] sector::ToBytesError),

	/// Unable to write sector
	#[error("Unable to write sector")]
	Write(#[source] std::io::Error),
}
