//! Type codes

// Imports
use dcb_bytes::Bytes;

/// A type code
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TypeCode {
	/// Boot record
	BootRecord,

	/// Primary
	Primary,

	/// Supplementary
	Supplementary,

	/// Volume partition
	VolumePartition,

	/// Set Terminator
	SetTerminator,

	/// Reserved
	Reserved(u8),
}

impl Bytes for TypeCode {
	type ByteArray = u8;
	type FromError = !;
	type ToError = !;

	fn from_bytes(byte: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let type_code = match byte {
			0 => Self::BootRecord,
			1 => Self::Primary,
			2 => Self::Supplementary,
			3 => Self::VolumePartition,
			0xFF => Self::SetTerminator,
			&byte => Self::Reserved(byte),
		};

		Ok(type_code)
	}

	fn to_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		*byte = match self {
			Self::BootRecord => 0,
			Self::Primary => 1,
			Self::Supplementary => 2,
			Self::VolumePartition => 3,
			Self::SetTerminator => 0xFF,
			Self::Reserved(byte) => *byte,
		};

		Ok(())
	}
}
