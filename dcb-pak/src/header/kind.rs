//! File kind

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// Kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// 3D model set.
	Model3DSet,

	/// Unknown 1
	Unknown1,

	/// Game script, `MSCD`
	GameScript,

	/// Animation2D
	Animation2D,

	/// File sub-header
	Unknown2,

	/// File contents
	FileContents,

	/// Audio `SEQ`
	AudioSeq,

	/// Audio `VH`
	AudioVh,

	/// Audio `VB`
	AudioVb,
}

/// Error type for [`Bytes::deserialize_bytes`]
#[derive(Debug, thiserror::Error)]
#[error("Invalid kind {_0}")]
pub struct DeserializeBytesError(pub u16);

impl Bytes for Kind {
	type ByteArray = [u8; 0x2];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let raw = LittleEndian::read_u16(bytes);
		let kind = match raw {
			0 => Self::Model3DSet,
			1 => Self::Unknown1,
			2 => Self::GameScript,
			3 => Self::Animation2D,
			4 => Self::Unknown2,
			5 => Self::FileContents,
			6 => Self::AudioSeq,
			7 => Self::AudioVh,
			8 => Self::AudioVb,
			_ => return Err(DeserializeBytesError(raw)),
		};

		Ok(kind)
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let raw = match self {
			Self::Model3DSet => 0,
			Self::Unknown1 => 1,
			Self::GameScript => 2,
			Self::Animation2D => 3,
			Self::Unknown2 => 4,
			Self::FileContents => 5,
			Self::AudioSeq => 6,
			Self::AudioVh => 7,
			Self::AudioVb => 8,
		};

		LittleEndian::write_u16(bytes, raw);

		Ok(())
	}
}
