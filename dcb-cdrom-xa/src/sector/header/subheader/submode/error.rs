//! Errors

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
/// and [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes).
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum BytesError {
	/// More than one of `Video`, `Audio` or `Data` were set
	#[error("More than one of `Video`, `Audio` or `Data` were set")]
	MoreThan1VideoAudioDataSet,
}
