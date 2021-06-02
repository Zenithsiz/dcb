//! Errors

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeBytesError {
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
