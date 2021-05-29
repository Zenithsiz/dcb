//! Errors

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Invalid tag
	#[error("Invalid tag {_0:#x}")]
	InvalidTag(u8),

	/// Invalid version
	#[error("Invalid version {_0:#x}")]
	InvalidVersion(u8),

	/// Unknown flag
	#[error("Unknown flag {_0:#010x}")]
	UnknownFlag(u32),
}
