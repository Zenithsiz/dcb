//! File kind

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// Kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Unknown 0
	Unknown0,

	/// Unknown 1
	Unknown1,

	/// Game script, `MSCD`
	GameScript,

	/// Animation2D
	Animation2D,

	/// File sub-header
	FileSubHeader,

	/// File contents
	FileContents,

	/// Audio `SEQ`
	AudioSeq,

	/// Audio `VH`
	AudioVh,

	/// Audio `VB`
	AudioVb,
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug, thiserror::Error)]
#[error("Invalid kind {_0}")]
pub struct FromBytesError(pub u16);

impl Bytes for Kind {
	type ByteArray = [u8; 0x2];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let raw = LittleEndian::read_u16(bytes);
		let kind = match raw {
			0 => Self::Unknown0,
			1 => Self::Unknown1,
			2 => Self::GameScript,
			3 => Self::Animation2D,
			4 => Self::FileSubHeader,
			5 => Self::FileContents,
			6 => Self::AudioSeq,
			7 => Self::AudioVh,
			8 => Self::AudioVb,
			_ => return Err(FromBytesError(raw)),
		};

		Ok(kind)
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let raw = match self {
			Self::Unknown0 => 0,
			Self::Unknown1 => 1,
			Self::GameScript => 2,
			Self::Animation2D => 3,
			Self::FileSubHeader => 4,
			Self::FileContents => 5,
			Self::AudioSeq => 6,
			Self::AudioVh => 7,
			Self::AudioVb => 8,
		};

		LittleEndian::write_u16(bytes, raw);

		Ok(())
	}
}
