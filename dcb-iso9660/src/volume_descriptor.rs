//! A volume descriptor

// Modules
pub mod boot;
pub mod error;
pub mod kind;
pub mod primary;

// Exports
pub use boot::BootRecordVolumeDescriptor;
pub use error::DeserializeBytesError;
pub use kind::DescriptorKind;
pub use primary::PrimaryVolumeDescriptor;

// Imports
use dcb_bytes::Bytes;
use dcb_util::array_split;

/// A volume descriptor
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum VolumeDescriptor {
	/// Boot record
	BootRecord(BootRecordVolumeDescriptor),

	/// Primary
	Primary(PrimaryVolumeDescriptor),

	/// Set terminator
	SetTerminator,
}

impl VolumeDescriptor {
	/// Returns the kind of descriptor this is
	#[must_use]
	pub const fn kind(&self) -> DescriptorKind {
		match self {
			Self::BootRecord { .. } => DescriptorKind::BootRecord,
			Self::Primary { .. } => DescriptorKind::Primary,
			Self::SetTerminator => DescriptorKind::SetTerminator,
		}
	}
}

impl VolumeDescriptor {
	/// Magic
	pub const MAGIC: [u8; 5] = *b"CD001";
	/// Version
	pub const VERSION: u8 = 0x1;
}

impl Bytes for VolumeDescriptor {
	type ByteArray = [u8; 0x800];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = array_split!(bytes,
			kind      :  0x1,
			magic     : [0x5],
			version   :  0x1,
			descriptor: [0x7f9],
		);

		// Get the descriptor kind
		let kind = DescriptorKind::deserialize_bytes(bytes.kind).into_ok();

		// If the magic is wrong, return Err
		if bytes.magic != &Self::MAGIC {
			return Err(DeserializeBytesError::InvalidMagic(*bytes.magic));
		}

		// If this isn't our version, return Err
		if bytes.version != &Self::VERSION {
			return Err(DeserializeBytesError::InvalidVersion(*bytes.version));
		}

		// Check the kind and parse the descriptor itself
		match kind {
			DescriptorKind::BootRecord => BootRecordVolumeDescriptor::deserialize_bytes(bytes.descriptor)
				.map(Self::BootRecord)
				.map_err(DeserializeBytesError::ParseBootRecord),
			DescriptorKind::Primary => PrimaryVolumeDescriptor::deserialize_bytes(bytes.descriptor)
				.map(Self::Primary)
				.map_err(DeserializeBytesError::ParsePrimary),
			DescriptorKind::SetTerminator => Ok(Self::SetTerminator),
			DescriptorKind::Supplementary | DescriptorKind::VolumePartition | DescriptorKind::Reserved(_) => {
				Err(DeserializeBytesError::CannotParseKind(kind))
			},
		}
	}

	fn serialize_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		todo!()
	}
}
