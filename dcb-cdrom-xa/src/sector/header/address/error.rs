//! Errors

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Invalid minute
	#[error("Invalid minute {_0:#x}")]
	InvalidMinute(u8),

	/// Invalid second
	#[error("Invalid second {_0:#x}")]
	InvalidSecond(u8),

	/// Invalid block
	#[error("Invalid block {_0:#x}")]
	InvalidBlock(u8),

	/// Out of range second
	#[error("Out of range second {_0:#x}")]
	OutOfRangeSecond(u8),

	/// Out of range block
	#[error("Out of range block {_0:#x}")]
	OutOfRangeBlock(u8),
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
#[allow(clippy::pub_enum_variant_names)] // It just happens they're all out of range, other errors may occur
pub enum ToBytesError {
	/// Out of range minute
	#[error("Out of range minute {_0:#x}")]
	OutOfRangeMinute(u8),
	
	/// Out of range second
	#[error("Out of range second {_0:#x}")]
	OutOfRangeSecond(u8),

	/// Out of range block
	#[error("Out of range block {_0:#x}")]
	OutOfRangeBlock(u8),
}
