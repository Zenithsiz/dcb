//! Errors

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Sync was wrong
	#[error("Sync was wrong, found {_0:?}")]
	Sync([u8; 0xc]),

	/// Invalid mode
	#[error("Invalid mode {_0:?}")]
	Mode(u8),
}
